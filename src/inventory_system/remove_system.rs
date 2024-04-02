use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteStorage};

use crate::{gamelog::Logger, CursedItem, Equipped, InBackpack, Name, WantsToRemoveItem};

pub struct ItemRemoveSystem;

impl<'a> System<'a> for ItemRemoveSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToRemoveItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CursedItem>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut wants_remove, mut equipped, mut backpack, player_entity, names, cursed) =
            data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            if cursed.get(to_remove.item).is_some() {
                Logger::new()
                    .white("You cannot unequip")
                    .cyan(&names.get(to_remove.item).unwrap().name)
                    .white("- it is cursed!")
                    .log();
                continue;
            }

            equipped.remove(to_remove.item);
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert backpack");
            if entity == *player_entity {
                Logger::new()
                    .white("You unequip")
                    .cyan(&names.get(to_remove.item).unwrap().name)
                    .log();
            }
        }

        wants_remove.clear();
    }
}
