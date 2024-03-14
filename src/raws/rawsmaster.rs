use super::{spawn_table_structs::SpawnTableEntry, Raws};
use crate::{
    components::{
        AreaOfEffect, BlocksTile, BlocksVisibility, Bystander, Confusion, Consumable, Door,
        EntryTrigger, EquipmentSlot, Equippable, Hidden, InflictsDamage, Item, MagicMapper,
        MeleeWeapon, Monster, Name, Position, ProvidesFood, ProvidesHealing, Quips, Ranged,
        SingleActivation, Vendor, Viewshed,
    },
    gamesystem::{attr_bonus, mana_at_level, npc_hp},
    random_table::RandomTable,
    Attribute, Attributes, Carnivore, Equipped, Herbivore, InBackpack, Initiative, IsSerialized,
    LightSource, LootTable, NaturalAttack, NaturalAttackDefense, Pool, Pools, Skill, Skills,
    WeaponAttribute, Wearable,
};
use regex::Regex;
use specs::{
    saveload::{MarkedBuilder, SimpleMarker},
    Builder, Entity, EntityBuilder, World, WorldExt,
};
use std::collections::{HashMap, HashSet};

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
    Equipped { by: Entity },
    Carried { by: Entity },
}

#[derive(Default)]
pub struct RawMaster {
    pub raws: Raws,
    pub item_index: HashMap<String, usize>,
    pub mob_index: HashMap<String, usize>,
    pub prop_index: HashMap<String, usize>,
    pub spawn_table: Vec<SpawnTableEntry>,
    pub loot_index: HashMap<String, usize>,
}

impl RawMaster {
    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        let mut used_names: HashSet<String> = HashSet::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            if used_names.contains(&item.name) {
                rltk::console::log(format!(
                    "WARNING -  duplicate item name in raws [{}]",
                    item.name
                ));
            }
            self.item_index.insert(item.name.clone(), i);
            used_names.insert(item.name.clone());
        }
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            if used_names.contains(&mob.name) {
                rltk::console::log(format!(
                    "WARNING -  duplicate mob name in raws [{}]",
                    mob.name
                ));
            }
            self.mob_index.insert(mob.name.clone(), i);
            used_names.insert(mob.name.clone());
        }
        for (i, prop) in self.raws.props.iter().enumerate() {
            if used_names.contains(&prop.name) {
                rltk::console::log(format!(
                    "WARNING -  duplicate prop name in raws [{}]",
                    prop.name
                ));
            }
            self.prop_index.insert(prop.name.clone(), i);
            used_names.insert(prop.name.clone());
        }

        for spawn in self.raws.spawn_table.iter() {
            if !used_names.contains(&spawn.name) {
                rltk::console::log(format!(
                    "WARNING - Spawn tables references unspecified entity {}",
                    spawn.name
                ));
            }
        }

        for (i, loot) in self.raws.loot_tables.iter().enumerate() {
            self.loot_index.insert(loot.name.clone(), i);
        }
    }
}

pub fn spawn_named_entity(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, ecs, key, pos);
    } else if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, ecs, key, pos);
    } else if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, ecs, key, pos);
    }

    None
}

pub fn spawn_named_item(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if !raws.item_index.contains_key(key) {
        return None;
    }
    let item_template = &raws.raws.items[raws.item_index[key]];
    let mut eb = ecs.create_entity().marked::<SimpleMarker<IsSerialized>>();

    // Spawn in the specified location
    eb = spawn_position(pos, eb, key, raws);

    // Renderable
    if let Some(renderable) = &item_template.renderable {
        eb = eb.with(get_renderable_component(renderable));
    }

    eb = eb.with(Name {
        name: item_template.name.clone(),
    });

    eb = eb.with(Item {});

    if let Some(weapon) = &item_template.weapon {
        eb = eb.with(Equippable {
            slot: EquipmentSlot::Melee,
        });

        let (n_dice, die_type, bonus) = parse_dice_string(&weapon.base_damage);
        let mut wpn = MeleeWeapon {
            attribute: WeaponAttribute::Might,
            damage_n_dice: n_dice,
            damage_die_type: die_type,
            damage_bonus: bonus,
            hit_bonus: weapon.hit_bonus,
        };

        match weapon.attribute.as_str() {
            "Quickness" => wpn.attribute = WeaponAttribute::Quickness,
            _ => wpn.attribute = WeaponAttribute::Might,
        }

        eb = eb.with(wpn);
    }

    if let Some(wearable) = &item_template.wearable {
        let slot = string_to_slot(&wearable.slot);
        eb = eb.with(Equippable { slot });
        eb = eb.with(Wearable {
            slot,
            armor_class: wearable.armor_class,
        });
    }

    if let Some(consumable) = &item_template.consumable {
        eb = eb.with(Consumable {});
        for (effect_name, effect_value) in consumable.effects.iter() {
            match effect_name.as_str() {
                "provides_healing" => {
                    eb = eb.with(ProvidesHealing {
                        heal_amount: effect_value.parse::<i32>().unwrap(),
                    })
                }
                "ranged" => {
                    eb = eb.with(Ranged {
                        range: effect_value.parse::<i32>().unwrap(),
                    })
                }
                "damage" => {
                    eb = eb.with(InflictsDamage {
                        damage: effect_value.parse::<i32>().unwrap(),
                    })
                }
                "area_of_effect" => {
                    eb = eb.with(AreaOfEffect {
                        radius: effect_value.parse::<i32>().unwrap(),
                    })
                }
                "confusion" => {
                    eb = eb.with(Confusion {
                        turns: effect_value.parse::<i32>().unwrap(),
                    })
                }
                "magic_mapping" => eb = eb.with(MagicMapper {}),
                "food" => eb = eb.with(ProvidesFood {}),
                _ => {
                    rltk::console::log(format!(
                        "Warning: consumable effect {} not implemented.",
                        effect_name
                    ));
                }
            }
        }
    }

    Some(eb.build())
}

pub fn spawn_named_mob(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if !raws.mob_index.contains_key(key) {
        return None;
    }
    let mob_template = &raws.raws.mobs[raws.mob_index[key]];
    let mut eb = ecs.create_entity().marked::<SimpleMarker<IsSerialized>>();

    // Spawn in the specified location
    eb = spawn_position(pos, eb, key, raws);

    // Renderable
    if let Some(renderable) = &mob_template.renderable {
        eb = eb.with(get_renderable_component(renderable));
    }

    match mob_template.ai_type.as_ref() {
        "melee" => eb = eb.with(Monster {}),
        "bystander" => eb = eb.with(Bystander {}),
        "vendor" => eb = eb.with(Vendor {}),
        "carnivore" => eb = eb.with(Carnivore {}),
        "herbivore" => eb = eb.with(Herbivore {}),
        _ => panic!("Unexpected AI Type for mob"),
    }

    eb = eb.with(Name {
        name: mob_template.name.clone(),
    });

    if let Some(quips) = &mob_template.quips {
        eb = eb.with(Quips {
            available: quips.clone(),
        });
    }

    if mob_template.blocks_tile {
        eb = eb.with(BlocksTile {});
    }

    eb = eb.with(Viewshed {
        visible_tiles: Vec::new(),
        range: mob_template.vision_range,
        dirty: true,
    });

    let mut attrs = Attributes {
        might: Attribute {
            base: 11,
            modifiers: 0,
            bonus: 0,
        },
        fitness: Attribute {
            base: 11,
            modifiers: 0,
            bonus: 0,
        },
        quickness: Attribute {
            base: 11,
            modifiers: 0,
            bonus: 0,
        },
        intelligence: Attribute {
            base: 11,
            modifiers: 0,
            bonus: 0,
        },
    };

    let mut mob_fitness = 11;
    let mut mob_int = 11;

    if let Some(m) = mob_template.attributes.might {
        attrs.might = Attribute {
            base: m,
            modifiers: 0,
            bonus: attr_bonus(m),
        }
    }

    // TODO(aalhendi): Refactor
    if let Some(f) = mob_template.attributes.fitness {
        attrs.fitness = Attribute {
            base: f,
            modifiers: 0,
            bonus: attr_bonus(f),
        };
        mob_fitness = f;
    }

    if let Some(q) = mob_template.attributes.quickness {
        attrs.quickness = Attribute {
            base: q,
            modifiers: 0,
            bonus: attr_bonus(q),
        }
    }

    if let Some(i) = mob_template.attributes.intelligence {
        attrs.intelligence = Attribute {
            base: i,
            modifiers: 0,
            bonus: attr_bonus(i),
        };
        mob_int = i;
    }

    eb = eb.with(attrs);

    let mut skills = Skills::default();
    for skill in [Skill::Melee, Skill::Defense, Skill::Magic] {
        skills.skills.insert(skill, 1);
    }
    if let Some(mobskills) = &mob_template.skills {
        for (skill_name, skill_value) in mobskills.iter() {
            match skill_name.as_str() {
                "Melee" => {
                    skills.skills.insert(Skill::Melee, *skill_value);
                }
                "Defense" => {
                    skills.skills.insert(Skill::Defense, *skill_value);
                }
                "Magic" => {
                    skills.skills.insert(Skill::Magic, *skill_value);
                }
                _ => {
                    rltk::console::log(format!("Unknown skill referenced: [{}]", skill_name));
                }
            }
        }
    }
    eb = eb.with(skills);

    let mob_level = if let Some(level) = mob_template.level {
        level
    } else {
        1
    };
    let mob_hp = npc_hp(mob_fitness, mob_level);
    let mob_mana = mana_at_level(mob_int, mob_level);

    let pools = Pools {
        level: mob_level,
        xp: 0,
        hit_points: Pool {
            current: mob_hp,
            max: mob_hp,
        },
        mana: Pool {
            current: mob_mana,
            max: mob_mana,
        },
    };
    eb = eb.with(pools);

    if let Some(nat) = &mob_template.natural {
        let attacks = nat
            .attacks
            .as_ref()
            .map(|attacks| {
                attacks
                    .iter()
                    .map(|nattack| {
                        let (n, d, b) = parse_dice_string(&nattack.damage);
                        NaturalAttack {
                            name: nattack.name.clone(),
                            hit_bonus: nattack.hit_bonus,
                            damage_n_dice: n,
                            damage_die_type: d,
                            damage_bonus: b,
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(Vec::new);

        eb = eb.with(NaturalAttackDefense {
            armor_class: nat.armor_class,
            attacks,
        });
    }

    // Loot
    if let Some(loot_table_name) = &mob_template.loot_table {
        eb = eb.with(LootTable {
            name: loot_table_name.clone(),
        });
    }

    if let Some(light) = &mob_template.light {
        eb = eb.with(LightSource {
            range: light.range,
            color: rltk::RGB::from_hex(&light.color).expect("Could not parse color"),
        });
    }

    eb = eb.with(Initiative { current: 2 });

    let new_mob = eb.build();

    // Wearables
    if let Some(wielding) = &mob_template.equipped {
        for tag in wielding.iter() {
            spawn_named_entity(raws, ecs, tag, SpawnType::Equipped { by: new_mob });
        }
    }

    Some(new_mob)
}

fn spawn_position<'a>(
    pos: SpawnType,
    new_entity: EntityBuilder<'a>,
    tag: &'a str,
    raws: &'a RawMaster,
) -> EntityBuilder<'a> {
    let eb = new_entity;

    // Spawn in the specified location
    match pos {
        SpawnType::AtPosition { x, y } => eb.with(Position { x, y }),
        SpawnType::Equipped { by } => {
            let slot = find_slot_for_equippable_item(tag, raws);
            eb.with(Equipped { owner: by, slot })
        }
        SpawnType::Carried { by } => eb.with(InBackpack { owner: by }),
    }
}

pub fn spawn_named_prop(
    raws: &RawMaster,
    ecs: &mut World,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if !raws.prop_index.contains_key(key) {
        return None;
    }
    let prop_template = &raws.raws.props[raws.prop_index[key]];

    let mut eb = ecs.create_entity().marked::<SimpleMarker<IsSerialized>>();

    // Spawn in the specified location
    eb = spawn_position(pos, eb, key, raws);

    // Renderable
    if let Some(renderable) = &prop_template.renderable {
        eb = eb.with(get_renderable_component(renderable));
    }

    eb = eb.with(Name {
        name: prop_template.name.clone(),
    });

    if let Some(hidden) = prop_template.hidden {
        if hidden {
            eb = eb.with(Hidden {})
        };
    }
    if let Some(blocks_tile) = prop_template.blocks_tile {
        if blocks_tile {
            eb = eb.with(BlocksTile {})
        };
    }
    if let Some(blocks_visibility) = prop_template.blocks_visibility {
        if blocks_visibility {
            eb = eb.with(BlocksVisibility {})
        };
    }
    if let Some(door_open) = prop_template.door_open {
        eb = eb.with(Door { open: door_open });
    }
    if let Some(entry_trigger) = &prop_template.entry_trigger {
        eb = eb.with(EntryTrigger {});
        for (effect_name, effect_value) in entry_trigger.effects.iter() {
            match effect_name.as_str() {
                "damage" => {
                    eb = eb.with(InflictsDamage {
                        damage: effect_value.parse::<i32>().unwrap(),
                    })
                }
                "single_activation" => eb = eb.with(SingleActivation {}),
                _ => {}
            }
        }
    }

    Some(eb.build())
}

fn get_renderable_component(
    renderable: &super::item_structs::Renderable,
) -> crate::components::Renderable {
    crate::components::Renderable {
        glyph: rltk::to_cp437(renderable.glyph.chars().next().unwrap()),
        fg: rltk::RGB::from_hex(&renderable.fg).expect("Invalid RGB"),
        bg: rltk::RGB::from_hex(&renderable.bg).expect("Invalid RGB"),
        render_order: renderable.order,
    }
}

pub fn get_spawn_table_for_depth(raws: &RawMaster, depth: i32) -> RandomTable {
    let available_options: Vec<&SpawnTableEntry> = raws
        .raws
        .spawn_table
        .iter()
        .filter(|a| depth >= a.min_depth && depth <= a.max_depth)
        .collect();

    let mut rt = RandomTable::new();
    for e in available_options.iter() {
        let mut weight = e.weight;
        if e.add_map_depth_to_weight == Some(true) {
            weight += depth;
        }
        rt = rt.add(e.name.clone(), weight);
    }

    rt
}

pub fn parse_dice_string(dice: &str) -> (i32, i32, i32) {
    lazy_static! {
        static ref DICE_RE: Regex =
            Regex::new(r"(\d+)d(\d+)([\+\-]\d+)?").expect("Could not create dice parsing regex.");
    }
    let mut n_dice = 1;
    let mut die_type = 4;
    let mut die_bonus = 0;
    for capture in DICE_RE.captures_iter(dice) {
        if let Some(group) = capture.get(1) {
            n_dice = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = capture.get(2) {
            die_type = group.as_str().parse::<i32>().expect("Not a digit");
        }
        if let Some(group) = capture.get(3) {
            die_bonus = group.as_str().parse::<i32>().expect("Not a digit");
        }
    }
    (n_dice, die_type, die_bonus)
}

fn find_slot_for_equippable_item(tag: &str, raws: &RawMaster) -> EquipmentSlot {
    if !raws.item_index.contains_key(tag) {
        panic!("Trying to equip an unknown item: {}", tag);
    }
    let item_index = raws.item_index[tag];
    let item = &raws.raws.items[item_index];
    if let Some(_wpn) = &item.weapon {
        return EquipmentSlot::Melee;
    } else if let Some(wearable) = &item.wearable {
        return string_to_slot(&wearable.slot);
    }
    panic!("Trying to equip {}, but it has no slot tag.", tag);
}

pub fn string_to_slot(slot: &str) -> EquipmentSlot {
    match slot {
        "Shield" => EquipmentSlot::Shield,
        "Head" => EquipmentSlot::Head,
        "Torso" => EquipmentSlot::Torso,
        "Legs" => EquipmentSlot::Legs,
        "Feet" => EquipmentSlot::Feet,
        "Hands" => EquipmentSlot::Hands,
        "Melee" => EquipmentSlot::Melee,
        _ => {
            rltk::console::log(format!("Warning: unknown equipment slot type [{}])", slot));
            EquipmentSlot::Melee
        }
    }
}

/// Check if table with name exists, and return None if it doesn't.
/// If it does exist, make a table of names and weights from the raw file information
/// Then roll to determine a randomly weighted result to return
pub fn get_item_drop(
    raws: &RawMaster,
    rng: &mut rltk::RandomNumberGenerator,
    table: &str,
) -> Option<String> {
    raws.loot_index.get(table).map(|table_index| {
        let available_options = &raws.raws.loot_tables[*table_index];
        let rt = available_options
            .drops
            .iter()
            .fold(RandomTable::new(), |acc, item| {
                acc.add(item.name.clone(), item.weight)
            });
        rt.roll(rng)
    })
}
