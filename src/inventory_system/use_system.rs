use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    gamelog::Gamelog, particle_system::ParticleBuilder, spatial, AreaOfEffect, Confusion,
    Consumable, EquipmentChanged, Equippable, Equipped, HungerClock, HungerState, IdentifiedItem,
    InBackpack, InflictsDamage, MagicMapper, Map, Name, Pools, Position, ProvidesFood,
    ProvidesHealing, RunState, SufferDamage, TownPortal, WantsToUseItem,
};

pub struct ItemUseSystem;

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Gamelog>,
        Entities<'a>,
        WriteStorage<'a, Consumable>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, Pools>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        WriteStorage<'a, Equipped>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, ProvidesFood>,
        WriteStorage<'a, HungerClock>,
        ReadStorage<'a, MagicMapper>,
        WriteExpect<'a, RunState>,
        WriteStorage<'a, EquipmentChanged>,
        ReadStorage<'a, TownPortal>,
        WriteStorage<'a, IdentifiedItem>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            consumables,
            mut wants_use,
            names,
            healers,
            damagers,
            mut pools,
            map,
            mut suffer_damage,
            aoe,
            mut confusers,
            mut equipped,
            equippable,
            mut backpack,
            mut particle_builder,
            positions,
            feeders,
            mut hunger_clocks,
            magic_mapper,
            mut runstate,
            mut dirty,
            town_portal,
            mut identified_item,
        ) = data;

        for (entity, useitem) in (&entities, &wants_use).join() {
            dirty
                .insert(entity, EquipmentChanged {})
                .expect("Unable to mark equipment changed");
            // Targeting
            let mut targets = Vec::new();
            match useitem.target {
                None => {
                    targets.push(*player_entity);
                }
                Some(target) => {
                    match aoe.get(useitem.item) {
                        None => {
                            // Single target in tile
                            let idx = map.xy_idx(target.x, target.y);
                            spatial::for_each_tile_content(idx, |mob| targets.push(mob));
                        }
                        Some(aoe) => {
                            // AoE
                            let mut blast_tiles = rltk::field_of_view(target, aoe.radius, &*map);
                            blast_tiles.retain(|p| {
                                p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                            });
                            for tile_idx in blast_tiles.iter() {
                                let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                spatial::for_each_tile_content(idx, |mob| targets.push(mob));
                                particle_builder.request(
                                    Position {
                                        x: tile_idx.x,
                                        y: tile_idx.y,
                                    },
                                    rltk::RGB::named(rltk::ORANGE),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('░'),
                                    200.0,
                                );
                            }
                        }
                    }
                }
            }

            // Identify
            if entity == *player_entity {
                identified_item
                    .insert(
                        entity,
                        IdentifiedItem {
                            name: names.get(useitem.item).unwrap().name.clone(),
                        },
                    )
                    .expect("Unable to insert");
            }

            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            let item_equippable = equippable.get(useitem.item);
            if let Some(can_equip) = item_equippable {
                let target_slot = can_equip.slot;
                let target = targets[0];

                // Remove any items the target has in the item's slot
                let mut to_unequip: Vec<Entity> = Vec::new();
                for (item_entity, already_equipped, name) in (&entities, &equipped, &names).join() {
                    if already_equipped.owner == target && already_equipped.slot == target_slot {
                        to_unequip.push(item_entity);
                        if target == *player_entity {
                            gamelog
                                .entries
                                .push(format!("You unequip {item_name}.", item_name = name.name));
                        }
                    }
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
                    gamelog.entries.push(format!(
                        "You equip {item_name}.",
                        item_name = names.get(useitem.item).unwrap().name
                    ));
                }
            }

            // Damaging Item
            if let Some(damager) = damagers.get(useitem.item) {
                for mob in targets.iter() {
                    if pools.get(*mob).is_some() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damager.damage, true);
                        if entity == *player_entity {
                            gamelog.entries.push(format!(
                                "You use {item_name} on {mob_name}, inflicting {amount} hp.",
                                amount = damager.damage,
                                mob_name = names.get(*mob).unwrap().name,
                                item_name = names.get(useitem.item).unwrap().name,
                            ));
                        }
                        if let Some(pos) = positions.get(*mob) {
                            particle_builder.request(
                                *pos,
                                rltk::RGB::named(rltk::RED),
                                rltk::RGB::named(rltk::BLACK),
                                rltk::to_cp437('‼'),
                                200.0,
                            );
                        }
                    }
                }
            }

            // Healing Item
            if let Some(healer) = healers.get(useitem.item) {
                for target in targets.iter() {
                    if let Some(stats) = pools.get_mut(*target) {
                        let amount = if stats.hit_points.current + healer.heal_amount
                            > stats.hit_points.max
                        {
                            stats.hit_points.max - stats.hit_points.current
                        } else {
                            healer.heal_amount
                        };
                        stats.hit_points.current = i32::min(
                            stats.hit_points.max,
                            stats.hit_points.current + healer.heal_amount,
                        );
                        if entity == *player_entity {
                            gamelog.entries.push(format!(
                                "You drink the {potion_name}, healing {amount} hp.",
                                potion_name = names.get(useitem.item).unwrap().name,
                            ));
                        }
                        if let Some(pos) = positions.get(*target) {
                            particle_builder.request(
                                *pos,
                                rltk::RGB::named(rltk::GREEN),
                                rltk::RGB::named(rltk::BLACK),
                                rltk::to_cp437('♥'),
                                200.0,
                            );
                        }
                    }
                }
            }

            // Confusion Item
            // map To avoid double borrow
            if let Some(turns) = confusers.get(useitem.item).map(|confuser| confuser.turns) {
                for mob in targets.iter() {
                    confusers
                        .insert(*mob, Confusion { turns })
                        .expect("Unable to insert status");
                    if entity == *player_entity {
                        gamelog.entries.push(format!(
                            "You use {item_name} on {mob_name}, confusing them.",
                            mob_name = names.get(*mob).unwrap().name,
                            item_name = names.get(useitem.item).unwrap().name,
                        ));
                    }
                    if let Some(pos) = positions.get(*mob) {
                        particle_builder.request(
                            *pos,
                            rltk::RGB::named(rltk::MAGENTA),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('?'),
                            200.0,
                        );
                    }
                }
            }

            // Edible Item
            if feeders.get(useitem.item).is_some() {
                if let Some(hc) = hunger_clocks.get_mut(targets[0]) {
                    hc.state = HungerState::WellFed;
                    hc.duration = 20;
                    gamelog.entries.push(format!(
                        "You eat the {item_name}.",
                        item_name = names.get(useitem.item).unwrap().name
                    ));
                }
            }

            // Magic Mapper Scroll
            if magic_mapper.get(useitem.item).is_some() {
                gamelog
                    .entries
                    .push("The map is revealed to you!".to_string());
                *runstate = RunState::MagicMapReveal { row: 0 };
            }

            // If its a town portal...
            if town_portal.get(useitem.item).is_some() {
                if map.depth == 1 {
                    gamelog
                        .entries
                        .push("You are already in town, so the scroll does nothing.".to_string());
                } else {
                    gamelog
                        .entries
                        .push("You are telported back to town!".to_string());
                    *runstate = RunState::TownPortal;
                }
            }

            if consumables.get(useitem.item).is_some() {
                entities.delete(useitem.item).expect("Delete failed");
            }
        }

        wants_use.clear();
    }
}
