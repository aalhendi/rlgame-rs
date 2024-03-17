use std::collections::HashMap;

use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{gamelog::Gamelog, Attributes, EquipmentChanged, Equipped, InBackpack, Item, Pools};

pub struct EncumbranceSystem;

impl<'a> System<'a> for EncumbranceSystem {
    type SystemData = (
        WriteStorage<'a, EquipmentChanged>,
        Entities<'a>,
        ReadStorage<'a, Item>,
        ReadStorage<'a, InBackpack>,
        ReadStorage<'a, Equipped>,
        WriteStorage<'a, Pools>,
        ReadStorage<'a, Attributes>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Gamelog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut equip_dirty,
            entities,
            items,
            backpacks,
            wielded,
            mut pools,
            attributes,
            player,
            mut gamelog,
        ) = data;

        if equip_dirty.is_empty() {
            return;
        }

        // Build the map of who needs updating
        let mut to_update = HashMap::new(); // (weight, intiative)
        for (entity, _dirty) in (&entities, &equip_dirty).join() {
            to_update.insert(entity, (0.0, 0.0));
        }

        // Remove all dirty statements
        equip_dirty.clear();

        // Total up equipped items
        for (item, equipped) in (&items, &wielded).join() {
            if let Some(totals) = to_update.get_mut(&equipped.owner) {
                totals.0 += item.weight_lbs;
                totals.1 += item.initiative_penalty;
            }
        }

        // Total up carried items
        for (item, carried) in (&items, &backpacks).join() {
            if let Some(totals) = to_update.get_mut(&carried.owner) {
                totals.0 += item.weight_lbs;
                totals.1 += item.initiative_penalty;
            }
        }

        // Apply the data to Pools
        for (entity, (weight, initiative)) in to_update.iter() {
            if let Some(pool) = pools.get_mut(*entity) {
                pool.total_weight = *weight;
                pool.total_initiative_penalty = *initiative;

                if let Some(attr) = attributes.get(*entity) {
                    let carry_capacity_lbs = (attr.might.base + attr.might.modifiers) * 15;
                    if pool.total_weight as i32 > carry_capacity_lbs {
                        // Overburdened
                        pool.total_initiative_penalty += 4.0;
                        if *entity == *player {
                            gamelog.entries.push(
                                "You are overburdened, and suffering an initiative penalty."
                                    .to_string(),
                            );
                        }
                    }
                }
            }
        }
    }
}
