use super::{
    gamelog::Gamelog, CombatStats, Consumable, InBackpack, InflictsDamage, Map, Name, Position,
    ProvidesHealing, SufferDamage, WantsToDropItem, WantsToPickupItem, WantsToUseItem,
};
use specs::prelude::*;

pub struct ItemCollectionSystem;

impl<'a> System<'a> for ItemCollectionSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Gamelog>,
        WriteStorage<'a, WantsToPickupItem>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

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

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!(
                    "You pick up the {}.",
                    names
                        .get(pickup.item)
                        .expect("Failed to get item name")
                        .name
                ));
            }
        }

        wants_pickup.clear();
    }
}

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
        WriteStorage<'a, CombatStats>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, SufferDamage>,
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
            mut combat_stats,
            map,
            mut suffer_damage,
        ) = data;

        for (entity, wants_use, stats) in (&entities, &wants_use, &mut combat_stats).join() {
            // Damaging Item
            if let Some(damager) = damagers.get(wants_use.item) {
                let target_pos = wants_use.target.unwrap();
                let idx = map.xy_idx(target_pos.x, target_pos.y);
                for mob in map.tile_content[idx].iter() {
                    SufferDamage::new_damage(&mut suffer_damage, *mob, damager.damage);
                    if entity == *player_entity {
                        gamelog.entries.push(format!(
                            "You use {item_name} on {mob_name}, inflicting {amount} hp.",
                            amount = damager.damage,
                            mob_name = names.get(*mob).unwrap().name,
                            item_name = names.get(wants_use.item).unwrap().name,
                        ));
                    }
                    // used_item = true;
                }
            }

            // Healing Item
            if let Some(healer) = healers.get(wants_use.item) {
                let amount = if stats.hp + healer.heal_amount > stats.max_hp {
                    stats.max_hp - stats.hp
                } else {
                    healer.heal_amount
                };
                stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                if entity == *player_entity {
                    gamelog.entries.push(format!(
                        "You drink the {potion_name}, healing {amount} hp.",
                        potion_name = names.get(wants_use.item).unwrap().name,
                    ));
                }
            }

            if consumables.get(wants_use.item).is_some() {
                entities.delete(wants_use.item).expect("Delete failed");
            }
        }

        wants_use.clear();
    }
}

pub struct ItemDropSystem;

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, Gamelog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let dropper_pos = *positions.get(entity).unwrap();

            positions
                .insert(to_drop.item, dropper_pos)
                .expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!(
                    "You drop the {item_name}.",
                    item_name = names.get(to_drop.item).unwrap().name
                ));
            }
        }
        wants_drop.clear();
    }
}
