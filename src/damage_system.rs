use crate::{
    gamesystem::{mana_at_level, player_hp_at_level},
    particle_system::ParticleBuilder,
    raws::{
        rawsmaster::{get_item_drop, spawn_named_item, SpawnType},
        RAWS,
    },
    Attributes, Equipped, InBackpack, LootTable, Pools,
};

use super::{gamelog::Gamelog, Map, Name, Player, Position, RunState, SufferDamage};
use rltk::Point;
use specs::prelude::*;

pub struct DamageSystem;

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, Pools>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, Gamelog>,
        WriteExpect<'a, ParticleBuilder>,
        ReadExpect<'a, Point>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut stats,
            mut damage,
            positions,
            mut map,
            entities,
            player_entity,
            attributes,
            mut log,
            mut particles,
            player_pos,
        ) = data;

        let mut xp_gain = 0;
        // Iterating through each entity that has both stats and damage components.
        for (entity, stats, damage) in (&entities, &mut stats, &damage).join() {
            for (dmg_amount, is_dmg_from_player) in &damage.amount {
                stats.hit_points.current -= dmg_amount;

                if stats.hit_points.current < 1 && *is_dmg_from_player {
                    xp_gain += stats.level * 100;
                }
            }

            // Inserting bloodstain if entity has Position component
            if let Some(pos) = positions.get(entity) {
                let idx = map.xy_idx(pos.x, pos.y);
                map.bloodstains.insert(idx);
            }
        }

        if xp_gain != 0 {
            let p_stats = stats.get_mut(*player_entity).unwrap();
            p_stats.xp += xp_gain;
            if p_stats.xp >= p_stats.level * 1000 {
                let player_attributes = attributes.get(*player_entity).unwrap();
                // We've gone up a level!
                p_stats.level += 1;
                let lvl_up_txt = format!("Congratulations, you are now level {}", p_stats.level);
                log.entries.push(lvl_up_txt);

                // Update stats
                p_stats.hit_points.max = player_hp_at_level(
                    player_attributes.fitness.base + player_attributes.fitness.modifiers,
                    p_stats.level,
                );
                p_stats.hit_points.current = p_stats.hit_points.max;
                p_stats.mana.max = mana_at_level(
                    player_attributes.intelligence.base + player_attributes.intelligence.modifiers,
                    p_stats.level,
                );
                p_stats.mana.current = p_stats.mana.max;

                // Particles
                for i in 0..10 {
                    if player_pos.y - i > 1 {
                        let particle_pos = Position {
                            x: player_pos.x,
                            y: player_pos.y - 1,
                        };
                        particles.request(
                            particle_pos,
                            rltk::RGB::named(rltk::GOLD),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('░'),
                            200.0,
                        );
                    }
                }
            }
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    // Using a scope to make the borrow checker happy
    {
        let pools = ecs.read_storage::<Pools>();
        let players = ecs.read_storage::<Player>();
        let entities = ecs.entities();
        let names = ecs.read_storage::<Name>();
        let mut log = ecs.write_resource::<Gamelog>();

        for (entity, stats) in (&entities, &pools).join() {
            if stats.hit_points.current < 1 {
                // Check if dead entity is player
                match players.get(entity) {
                    None => {
                        if let Some(victim_name) = names.get(entity) {
                            log.entries
                                .push(format!("{name} is dead", name = &victim_name.name))
                        }
                        dead.push(entity)
                    }
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                    }
                }
            }
        }
    }

    // Drop everything held by dead people
    let mut to_spawn: Vec<(String, Position)> = Vec::new();
    {
        let mut to_drop: Vec<(Entity, Position)> = Vec::new();
        let entities = ecs.entities();
        let mut equipped = ecs.write_storage::<Equipped>();
        let mut carried = ecs.write_storage::<InBackpack>();
        let mut positions = ecs.write_storage::<Position>();
        let loot_tables = ecs.read_storage::<LootTable>();
        let mut rng = ecs.write_resource::<rltk::RandomNumberGenerator>();

        for victim in dead.iter() {
            let pos = positions.get(*victim);

            // Drop equipped items
            for (entity, equipped) in (&entities, &equipped).join() {
                if equipped.owner == *victim {
                    if let Some(pos) = pos {
                        to_drop.push((entity, *pos));
                    }
                }
            }

            // Drop carried items
            for (entity, backpack) in (&entities, &carried).join() {
                if backpack.owner == *victim {
                    if let Some(pos) = pos {
                        to_drop.push((entity, *pos));
                    }
                }
            }

            // Drop loot items
            if let Some(table) = loot_tables.get(*victim) {
                let drop_finder = get_item_drop(&RAWS.lock().unwrap(), &mut rng, &table.name);
                if let Some(tag) = drop_finder {
                    if let Some(pos) = pos {
                        to_spawn.push((tag, *pos));
                    }
                }
            }
        }

        for (entity, drop_pos) in to_drop {
            equipped.remove(entity);
            carried.remove(entity);
            positions
                .insert(entity, drop_pos)
                .expect("Unable to insert position");
        }
    }

    for (key_name, spawn_pos) in to_spawn.iter() {
        spawn_named_item(
            &RAWS.lock().unwrap(),
            ecs,
            key_name,
            SpawnType::AtPosition {
                x: spawn_pos.x,
                y: spawn_pos.y,
            },
        );
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
