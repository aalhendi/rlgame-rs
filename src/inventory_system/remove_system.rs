use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{gamelog::Gamelog, Equipped, InBackpack, Name, WantsToRemoveItem};

pub struct ItemRemoveSystem;

impl<'a> System<'a> for ItemRemoveSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToRemoveItem>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, Gamelog>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_remove,
            mut equipped,
            mut backpack,
            mut gamelog,
            player_entity,
            names,
        ) = data;

        for (entity, to_remove) in (&entities, &wants_remove).join() {
            equipped.remove(to_remove.item);
            backpack
                .insert(to_remove.item, InBackpack { owner: entity })
                .expect("Unable to insert backpack");
            if entity == *player_entity {
                gamelog.entries.push(format!(
                    "You unequip the {item_name}.",
                    item_name = names.get(to_remove.item).unwrap().name
                ));
            }
        }

        wants_remove.clear();
    }
}
