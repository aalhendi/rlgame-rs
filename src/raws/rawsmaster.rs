use super::{faction_structs::Reaction, spawn_table_structs::SpawnTableEntry, Raws, RAWS};
use crate::{
    components::{
        AreaOfEffect, BlocksTile, BlocksVisibility, Confusion, Consumable, Door, EntryTrigger,
        EquipmentSlot, Equippable, Hidden, InflictsDamage, Item, MagicMapper, MeleeWeapon, Name,
        Position, ProvidesFood, ProvidesHealing, Quips, Ranged, SingleActivation, Viewshed,
    },
    dungeon::MasterDungeonMap,
    gamesystem::{attr_bonus, mana_at_level, npc_hp},
    random_table::{MasterTable, RandomTable},
    AlwaysTargetsSelf, Attribute, AttributeBonus, Attributes, CursedItem, DamageOverTime, Duration,
    Equipped, Faction, InBackpack, Initiative, IsSerialized, LightSource, LootTable, MagicItem,
    MagicItemClass, MoveMode, Movement, NaturalAttack, NaturalAttackDefense, ObfuscatedName,
    OnDeath, Pool, Pools, ProvidesMana, ProvidesRemoveCurse, Skill, Skills, Slow,
    SpawnParticleBurst, SpawnParticleLine, SpecialAbilities, SpecialAbility, SpellTemplate,
    TeachesSpell, TileSize, TownPortal, Vendor, WeaponAttribute, Wearable,
};
use regex::Regex;
use specs::{
    saveload::{MarkedBuilder, SimpleMarker},
    Builder, Entities, Entity, EntityBuilder, Join, ReadStorage, World, WorldExt,
};
use std::collections::{HashMap, HashSet};

macro_rules! apply_effects {
    ( $effects:expr, $eb:expr ) => {
        for effect in $effects.iter() {
            let effect_name = effect.0.as_str();
            match effect_name {
                "provides_healing" => {
                    $eb = $eb.with(ProvidesHealing {
                        heal_amount: effect.1.parse::<i32>().unwrap(),
                    })
                }
                "provides_mana" => {
                    $eb = $eb.with(ProvidesMana {
                        mana_amount: effect.1.parse::<i32>().unwrap(),
                    })
                }
                "ranged" => {
                    $eb = $eb.with(Ranged {
                        range: effect.1.parse::<i32>().unwrap(),
                    })
                }
                "damage" => {
                    $eb = $eb.with(InflictsDamage {
                        damage: effect.1.parse::<i32>().unwrap(),
                    })
                }
                "area_of_effect" => {
                    $eb = $eb.with(AreaOfEffect {
                        radius: effect.1.parse::<i32>().unwrap(),
                    })
                }
                "confusion" => {
                    $eb = $eb.with(Confusion {});
                    $eb = $eb.with(Duration {
                        turns: effect.1.parse::<i32>().unwrap(),
                    });
                }
                "magic_mapping" => $eb = $eb.with(MagicMapper {}),
                "town_portal" => $eb = $eb.with(TownPortal {}),
                "food" => $eb = $eb.with(ProvidesFood {}),
                "single_activation" => $eb = $eb.with(SingleActivation {}),
                "particle_line" => $eb = $eb.with(parse_particle_line(&effect.1)),
                "particle" => $eb = $eb.with(parse_particle(&effect.1)),
                "remove_curse" => $eb = $eb.with(ProvidesRemoveCurse {}),
                "target_self" => $eb = $eb.with(AlwaysTargetsSelf {}),
                "teach_spell" => {
                    $eb = $eb.with(TeachesSpell {
                        spell: effect.1.to_string(),
                    })
                }
                "slow" => {
                    $eb = $eb.with(Slow {
                        initiative_penalty: effect.1.parse::<f32>().unwrap(),
                    })
                }
                "damage_over_time" => {
                    $eb = $eb.with(DamageOverTime {
                        damage: effect.1.parse::<i32>().unwrap(),
                    })
                }
                _ => rltk::console::log(format!(
                    "Warning: consumable effect {} not implemented.",
                    effect_name
                )),
            }
        }
    };
}

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
    faction_index: HashMap<String, HashMap<String, Reaction>>,
    spell_index: HashMap<String, usize>,
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

        // iterates through all factions, then through reactions to other factions
        // builds HashMap of reactions to each faction stored in faction_index
        for faction in self.raws.faction_table.iter() {
            let mut reactions: HashMap<String, Reaction> = HashMap::new();
            for other in faction.responses.iter() {
                reactions.insert(
                    other.0.clone(),
                    match other.1.as_str() {
                        "ignore" => Reaction::Ignore,
                        "flee" => Reaction::Flee,
                        _ => Reaction::Attack,
                    },
                );
            }
            self.faction_index.insert(faction.name.clone(), reactions);
        }

        for (i, spell) in self.raws.spells.iter().enumerate() {
            self.spell_index.insert(spell.name.clone(), i);
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

pub fn spawn_all_spells(ecs: &mut World) {
    let raws = &RAWS.lock().unwrap();
    for spell in raws.raws.spells.iter() {
        spawn_named_spell(raws, ecs, &spell.name);
    }
}

pub fn spawn_named_spell(raws: &RawMaster, ecs: &mut World, key: &str) -> Option<Entity> {
    if !raws.spell_index.contains_key(key) {
        return None;
    }

    let spell_template = &raws.raws.spells[raws.spell_index[key]];

    let mut eb = ecs.create_entity().marked::<SimpleMarker<IsSerialized>>();
    eb = eb.with(SpellTemplate {
        mana_cost: spell_template.mana_cost,
    });
    eb = eb.with(Name {
        name: spell_template.name.clone(),
    });
    apply_effects!(spell_template.effects, eb);

    Some(eb.build())
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
    let dm = ecs.fetch::<MasterDungeonMap>();
    let scroll_names = dm.scroll_mappings.clone();
    let potion_names = dm.potion_mappings.clone();
    let identified = dm.identified_items.clone();
    std::mem::drop(dm);
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

    eb = eb.with(Item {
        initiative_penalty: item_template.initiative_penalty.unwrap_or(0.0),
        weight_lbs: item_template.weight_lbs.unwrap_or(0.0),
        base_value: item_template.base_value.unwrap_or(0.0),
    });

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
            proc_chance: weapon.proc_chance,
            proc_target: weapon.proc_target.clone(),
        };

        match weapon.attribute.as_str() {
            "Quickness" => wpn.attribute = WeaponAttribute::Quickness,
            _ => wpn.attribute = WeaponAttribute::Might,
        }

        eb = eb.with(wpn);
        if let Some(proc_effects) = &weapon.proc_effects {
            apply_effects!(proc_effects, eb);
        }
    }

    if let Some(wearable) = &item_template.wearable {
        let slot = string_to_slot(&wearable.slot);
        eb = eb.with(Equippable { slot });
        eb = eb.with(Wearable {
            slot,
            armor_class: wearable.armor_class,
        });
    }

    if let Some(magic) = &item_template.magic {
        let class = match magic.class.as_str() {
            "rare" => MagicItemClass::Rare,
            "legendary" => MagicItemClass::Legendary,
            _ => MagicItemClass::Common,
        };
        eb = eb.with(MagicItem { class });

        if !identified.contains(&item_template.name) {
            match magic.naming.as_str() {
                "scroll" => {
                    eb = eb.with(ObfuscatedName {
                        name: scroll_names[&item_template.name].clone(),
                    });
                }
                "potion" => {
                    eb = eb.with(ObfuscatedName {
                        name: potion_names[&item_template.name].clone(),
                    });
                }
                _ => {
                    eb = eb.with(ObfuscatedName {
                        name: magic.naming.clone(),
                    })
                }
            }
        }

        if magic.cursed.is_some_and(|c| c) {
            eb = eb.with(CursedItem {});
        }
    }

    if let Some(ab) = &item_template.attributes {
        eb = eb.with(AttributeBonus {
            might: ab.might,
            fitness: ab.fitness,
            quickness: ab.quickness,
            intelligence: ab.intelligence,
        });
    }

    if let Some(consumable) = &item_template.consumable {
        let max_charges = consumable.charges.unwrap_or(1);
        eb = eb.with(Consumable {
            max_charges,
            charges: max_charges,
        });
        apply_effects!(consumable.effects, eb);
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
        if renderable.x_size.is_some() || renderable.y_size.is_some() {
            eb = eb.with(TileSize {
                x: renderable.x_size.unwrap_or(1),
                y: renderable.y_size.unwrap_or(1),
            });
        }
    }

    match mob_template.movement.as_ref() {
        "random" => {
            eb = eb.with(MoveMode {
                mode: Movement::Random,
            })
        }
        "random_waypoint" => {
            eb = eb.with(MoveMode {
                mode: Movement::RandomWaypoint { path: None },
            })
        }

        _ => {
            eb = eb.with(MoveMode {
                mode: Movement::Static,
            })
        }
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

    let mut rng = rltk::RandomNumberGenerator::new();
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
        total_weight: 0.0,
        total_initiative_penalty: 0.0,
        god_mode: false,
        gold: mob_template
            .gold
            .as_ref()
            .map(|gold| {
                let (n, d, b) = parse_dice_string(gold);
                (rng.roll_dice(n, d) + b) as f32
            })
            .unwrap_or(0.0),
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

    if let Some(faction) = &mob_template.faction {
        eb = eb.with(Faction {
            name: faction.clone(),
        });
    } else {
        eb = eb.with(Faction {
            name: "Mindless".to_string(),
        })
    }

    // TODO(aalhendi): Mindless and vendor might not be the best combo
    if let Some(vendor) = &mob_template.vendor {
        eb = eb.with(Vendor {
            categories: vendor.clone(),
        });
    }

    let special_abilities = mob_template
        .abilities
        .as_ref()
        .map(|ability_list| SpecialAbilities {
            abilities: ability_list
                .iter()
                .map(|ability| SpecialAbility {
                    chance: ability.chance,
                    spell: ability.spell.clone(),
                    range: ability.range,
                    min_range: ability.min_range,
                })
                .collect(),
        });

    eb = eb.with(special_abilities.unwrap_or_default());

    let death_abilities = mob_template.on_death.as_ref().map(|ability_list| OnDeath {
        abilities: ability_list
            .iter()
            .map(|ability| SpecialAbility {
                chance: ability.chance,
                spell: ability.spell.clone(),
                range: ability.range,
                min_range: ability.min_range,
            })
            .collect(),
    });

    eb = eb.with(death_abilities.unwrap_or_default());

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
        apply_effects!(entry_trigger.effects, eb);
    }

    if let Some(light) = &prop_template.light {
        eb = eb.with(LightSource {
            range: light.range,
            color: rltk::RGB::from_hex(&light.color).expect("Bad color"),
        });
        eb = eb.with(Viewshed {
            range: light.range,
            dirty: true,
            visible_tiles: Vec::new(),
        });
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

pub fn get_spawn_table_for_depth(raws: &RawMaster, depth: i32) -> MasterTable {
    let available_options: Vec<&SpawnTableEntry> = raws
        .raws
        .spawn_table
        .iter()
        .filter(|a| depth >= a.min_depth && depth <= a.max_depth)
        .collect();

    let mut mt = MasterTable::new();
    for e in available_options.iter() {
        let mut weight = e.weight;
        if e.add_map_depth_to_weight == Some(true) {
            weight += depth;
        }
        mt.add(e.name.clone(), weight, raws);
    }

    mt
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
    if !raws.loot_index.contains_key(table) {
        return None;
    }

    let mut rt = RandomTable::new();
    let available_options = &raws.raws.loot_tables[raws.loot_index[table]];
    for item in available_options.drops.iter() {
        rt.add(item.name.clone(), item.weight);
    }
    let result = rt.roll(rng);
    Some(result)
}

pub fn faction_reaction(my_faction: &str, their_faction: &str, raws: &RawMaster) -> Reaction {
    raws.faction_index
        .get(my_faction)
        .and_then(|reactions| {
            reactions
                .get(their_faction)
                .or_else(|| reactions.get("Default"))
        })
        .copied()
        //default to Ignore (shouldn't happen, since we default to Mindless)
        .unwrap_or(Reaction::Ignore)
}

pub fn get_vendor_items(categories: &[String], raws: &RawMaster) -> Vec<(String, f32)> {
    raws.raws
        .items
        .iter()
        .filter_map(|item| match (&item.vendor_category, item.base_value) {
            (Some(cat), Some(base_value)) if categories.contains(cat) => {
                Some((item.name.clone(), base_value))
            }
            _ => None,
        })
        .collect::<Vec<_>>()
}

pub fn get_scroll_tags() -> Vec<String> {
    let raws = &RAWS.lock().unwrap();
    let mut result = Vec::new();

    for item in raws.raws.items.iter() {
        if item
            .magic
            .as_ref()
            .is_some_and(|magic| magic.naming == "scroll")
        {
            result.push(item.name.clone());
        }
    }

    result
}

pub fn get_potion_tags() -> Vec<String> {
    let raws = &RAWS.lock().unwrap();
    let mut result = Vec::new();

    for item in raws.raws.items.iter() {
        if item
            .magic
            .as_ref()
            .is_some_and(|magic| magic.naming == "potion")
        {
            result.push(item.name.clone());
        }
    }

    result
}

pub fn is_tag_magic(tag: &str) -> bool {
    let raws = &RAWS.lock().unwrap();
    if raws.item_index.contains_key(tag) {
        let item_template = &raws.raws.items[raws.item_index[tag]];
        item_template.magic.is_some()
    } else {
        false
    }
}

fn parse_particle_line(n: &str) -> SpawnParticleLine {
    let tokens: Vec<_> = n.split(';').collect();
    SpawnParticleLine {
        glyph: rltk::to_cp437(tokens[0].chars().next().unwrap()),
        color: rltk::RGB::from_hex(tokens[1]).expect("Bad RGB"),
        lifetime_ms: tokens[2].parse::<f32>().unwrap(),
    }
}

fn parse_particle(n: &str) -> SpawnParticleBurst {
    let tokens: Vec<_> = n.split(';').collect();
    SpawnParticleBurst {
        glyph: rltk::to_cp437(tokens[0].chars().next().unwrap()),
        color: rltk::RGB::from_hex(tokens[1]).expect("Bad RGB"),
        lifetime_ms: tokens[2].parse::<f32>().unwrap(),
    }
}

pub fn find_spell_entity(ecs: &World, name: &str) -> Option<Entity> {
    let names = ecs.read_storage::<Name>();
    let spell_templates = ecs.read_storage::<SpellTemplate>();
    let entities = ecs.entities();

    for (entity, sname, _template) in (&entities, &names, &spell_templates).join() {
        if name == sname.name {
            return Some(entity);
        }
    }
    None
}

// In-System version
pub fn find_spell_entity_by_name(
    name: &str,
    names: &ReadStorage<Name>,
    spell_templates: &ReadStorage<SpellTemplate>,
    entities: &Entities,
) -> Option<Entity> {
    for (entity, sname, _template) in (entities, names, spell_templates).join() {
        if name == sname.name {
            return Some(entity);
        }
    }
    None
}

pub enum SpawnTableType {
    Item,
    Mob,
    Prop,
}

pub fn spawn_type_by_name(raws: &RawMaster, key: &str) -> SpawnTableType {
    if raws.item_index.contains_key(key) {
        SpawnTableType::Item
    } else if raws.mob_index.contains_key(key) {
        SpawnTableType::Mob
    } else {
        SpawnTableType::Prop
    }
}
