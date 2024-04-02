use specs::{Entity, Join, ReadExpect, ReadStorage, System, WriteStorage};

use crate::{
    dungeon::MasterDungeonMap, gamelog::Logger, EquipmentChanged, InBackpack, MagicItem, Name,
    ObfuscatedName, Position, WantsToPickupItem,
};

use super::obfuscate_name;

pub struct ItemCollectionSystem;

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, MagicItem>,
        ReadStorage<'a, ObfuscatedName>,
        ReadExpect<'a, MasterDungeonMap>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut wants_pickup,
            mut positions,
            names,
            mut backpack,
            mut dirty,
            magic_items,
            obfuscated_names,
            dm,
        ) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    InBackpack {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");
            dirty
                .insert(pickup.item, EquipmentChanged {})
                .expect("Unable to mark EquipmentChanged");

            if pickup.collected_by == *player_entity {
                Logger::new()
                    .white("You pick up the")
                    .cyan(obfuscate_name(
                        pickup.item,
                        &names,
                        &magic_items,
                        &obfuscated_names,
                        &dm,
                    ))
                    .log();
            }
        }

        wants_pickup.clear();
    }
}
