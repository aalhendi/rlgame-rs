use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    gamelog::Gamelog, CursedItem, EquipmentChanged, Equippable, Equipped, IdentifiedItem,
    InBackpack, Name, WantsToUseItem,
};

pub struct ItemEquipOnUse;

impl<'a> System<'a> for ItemEquipOnUse {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Gamelog>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteStorage<'a, EquipmentChanged>,
        WriteStorage<'a, IdentifiedItem>,
        ReadStorage<'a, CursedItem>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_use,
            names,
            equippable,
            mut equipped,
            mut backpack,
            mut dirty,
            mut identified_item,
            cursed,
        ) = data;

        let mut remove_use = Vec::new();
        for (target, useitem) in (&entities, &wants_use).join() {
            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            let can_equip = equippable.get(useitem.item);
            if can_equip.is_none() {
                continue;
            }
            let target_slot = can_equip.unwrap().slot;
            let mut can_equip = true;

            // Remove any items the target has in the item's slot
            let mut to_unequip = Vec::new();
            for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                if already_equipped.owner == target && already_equipped.slot == target_slot {
                    if cursed.get(item_entity).is_some() {
                        can_equip = false;
                        gamelog
                            .entries
                            .push(format!("You cannot unequip {}, it is cursed.", name.name));
                    } else {
                        to_unequip.push(item_entity);
                        if target == *player_entity {
                            gamelog.entries.push(format!("You unequip {}.", name.name));
                        }
                    }
                }
            }
            if can_equip {
                // Identify the item
                if target == *player_entity {
                    identified_item
                        .insert(
                            target,
                            IdentifiedItem {
                                name: names.get(useitem.item).unwrap().name.clone(),
                            },
                        )
                        .expect("Unable to insert");
                }

                for item in to_unequip.iter() {
                    equipped.remove(*item);
                    backpack
                        .insert(*item, InBackpack { owner: target })
                        .expect("Unable to insert backpack entry");
                }

                // Wield the item
                equipped
                    .insert(
                        useitem.item,
                        Equipped {
                            owner: target,
                            slot: target_slot,
                        },
                    )
                    .expect("Unable to insert equipped component");
                backpack.remove(useitem.item);
                if target == *player_entity {
                    let entry = format!("You equip {}.", names.get(useitem.item).unwrap().name);
                    gamelog.entries.push(entry);
                }
            }

            // Done with item
            remove_use.push(target);
        }

        remove_use.iter().for_each(|e| {
            dirty
                .insert(*e, EquipmentChanged {})
                .expect("Unable to insert");
            wants_use.remove(*e).expect("Unable to remove");
        });
    }
}
