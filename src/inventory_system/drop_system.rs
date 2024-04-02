use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteStorage};

use crate::{
    dungeon::MasterDungeonMap, gamelog::Logger, EquipmentChanged, InBackpack, MagicItem, Name,
    ObfuscatedName, Position, WantsToDropItem,
};

use super::obfuscate_name;

pub struct ItemDropSystem;

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, MagicItem>,
        ReadStorage<'a, ObfuscatedName>,
        ReadExpect<'a, MasterDungeonMap>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack,
            mut dirty,
            magic_items,
            obfuscated_names,
            dm,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let dropper_pos = *positions.get(entity).unwrap();

            positions
                .insert(to_drop.item, dropper_pos)
                .expect("Unable to insert position");
            backpack.remove(to_drop.item);
            dirty
                .insert(entity, EquipmentChanged {})
                .expect("Unable to mark equipment changed");

            if entity == *player_entity {
                Logger::new()
                    .white("You drop the")
                    .cyan(obfuscate_name(
                        to_drop.item,
                        &names,
                        &magic_items,
                        &obfuscated_names,
                        &dm,
                    ))
                    .log();
            }
        }
        wants_drop.clear();
    }
}
