use specs::{Entities, Join, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{dungeon::MasterDungeonMap, raws::rawsmaster::is_tag_magic, IdentifiedItem, Item, Name, ObfuscatedName, Player};

pub struct ItemIdentificationSystem;

impl<'a> System<'a> for ItemIdentificationSystem {
    type SystemData = (
        ReadStorage<'a, Player>,
        WriteStorage<'a, IdentifiedItem>,
        WriteExpect<'a, MasterDungeonMap>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, ObfuscatedName>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player, mut identified, mut dm, items, names, mut obfuscated_names, entities) = data;

        for (_p, id) in (&player, &identified).join() {
            if !dm.identified_items.contains(&id.name) && is_tag_magic(&id.name) {
                dm.identified_items.insert(id.name.clone());

                for (entity, _item, name) in (&entities, &items, &names).join() {
                    if name.name == id.name {
                        obfuscated_names.remove(entity);
                    }
                }
            }
        }

        // Clean up
        identified.clear();
    }
}
