use super::Raws;
use crate::components::{
    AreaOfEffect, BlocksTile, BlocksVisibility, CombatStats, Confusion, Consumable, DefenseBonus,
    Door, EntryTrigger, EquipmentSlot, Equippable, Hidden, InflictsDamage, Item, MagicMapper,
    MeleePowerBonus, Monster, Name, Position, ProvidesFood, ProvidesHealing, Ranged,
    SingleActivation, Viewshed,
};
use specs::{Builder, Entity, EntityBuilder};
use std::collections::HashMap;

pub enum SpawnType {
    AtPosition { x: i32, y: i32 },
}

#[derive(Default)]
pub struct RawMaster {
    pub raws: Raws,
    pub item_index: HashMap<String, usize>,
    pub mob_index: HashMap<String, usize>,
    pub prop_index: HashMap<String, usize>,
}

impl RawMaster {
    pub fn load(&mut self, raws: Raws) {
        self.raws = raws;
        self.item_index = HashMap::new();
        for (i, item) in self.raws.items.iter().enumerate() {
            self.item_index.insert(item.name.clone(), i);
        }
        for (i, mob) in self.raws.mobs.iter().enumerate() {
            self.mob_index.insert(mob.name.clone(), i);
        }
        for (i, prop) in self.raws.props.iter().enumerate() {
            self.prop_index.insert(prop.name.clone(), i);
        }
    }
}

pub fn spawn_named_entity(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if raws.item_index.contains_key(key) {
        return spawn_named_item(raws, new_entity, key, pos);
    } else if raws.mob_index.contains_key(key) {
        return spawn_named_mob(raws, new_entity, key, pos);
    } else if raws.prop_index.contains_key(key) {
        return spawn_named_prop(raws, new_entity, key, pos);
    }

    None
}

pub fn spawn_named_item(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if !raws.item_index.contains_key(key) {
        return None;
    }
    let item_template = &raws.raws.items[raws.item_index[key]];
    let mut eb = new_entity;

    // Spawn in the specified location
    eb = spawn_position(pos, eb);

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
        eb = eb.with(MeleePowerBonus {
            amount: weapon.power_bonus,
        });
    }

    if let Some(shield) = &item_template.shield {
        eb = eb.with(Equippable {
            slot: EquipmentSlot::Shield,
        });
        eb = eb.with(DefenseBonus {
            amount: shield.defense_bonus,
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
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if !raws.mob_index.contains_key(key) {
        return None;
    }
    let mob_template = &raws.raws.mobs[raws.mob_index[key]];
    let mut eb = new_entity;

    // Spawn in the specified location
    eb = spawn_position(pos, eb);

    // Renderable
    if let Some(renderable) = &mob_template.renderable {
        eb = eb.with(get_renderable_component(renderable));
    }

    eb = eb.with(Name {
        name: mob_template.name.clone(),
    });

    eb = eb.with(Monster {});
    if mob_template.blocks_tile {
        eb = eb.with(BlocksTile {});
    }
    eb = eb.with(CombatStats {
        max_hp: mob_template.stats.max_hp,
        hp: mob_template.stats.hp,
        power: mob_template.stats.power,
        defense: mob_template.stats.defense,
    });
    eb = eb.with(Viewshed {
        visible_tiles: Vec::new(),
        range: mob_template.vision_range,
        dirty: true,
    });

    Some(eb.build())
}

fn spawn_position(pos: SpawnType, new_entity: EntityBuilder) -> EntityBuilder {
    let mut eb = new_entity;

    // Spawn in the specified location
    match pos {
        SpawnType::AtPosition { x, y } => {
            eb = eb.with(Position { x, y });
        }
    }

    eb
}

pub fn spawn_named_prop(
    raws: &RawMaster,
    new_entity: EntityBuilder,
    key: &str,
    pos: SpawnType,
) -> Option<Entity> {
    if !raws.prop_index.contains_key(key) {
        return None;
    }
    let prop_template = &raws.raws.props[raws.prop_index[key]];

    let mut eb = new_entity;

    // Spawn in the specified location
    eb = spawn_position(pos, eb);

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
