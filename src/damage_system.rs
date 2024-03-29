use crate::{
    effects::{add_effect, targetting::aoe_tiles, EffectType, Targets},
    raws::{
        rawsmaster::{find_spell_entity, get_item_drop, spawn_named_item, SpawnType},
        RAWS,
    },
    AreaOfEffect, Equipped, InBackpack, LootTable, Map, OnDeath, Pools,
};

use super::{gamelog::Gamelog, Name, Player, Position, RunState};
use specs::prelude::*;

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
                if let Some(tag) = get_item_drop(&RAWS.lock().unwrap(), &mut rng, &table.name) {
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

    for victim in dead.iter() {
        let death_effects = ecs.read_storage::<OnDeath>();
        if let Some(death_effect) = death_effects.get(*victim) {
            let mut rng = ecs.fetch_mut::<rltk::RandomNumberGenerator>();
            for effect in death_effect.abilities.iter() {
                if rng.roll_dice(1, 100) > (effect.chance * 100.0) as i32 {
                    continue;
                }
                let map = ecs.fetch::<Map>();
                if ecs.read_storage::<Position>().get(*victim).is_none() {
                    continue;
                }
                let pos = *ecs.read_storage::<Position>().get(*victim).unwrap();
                let spell_entity = find_spell_entity(ecs, &effect.spell).unwrap();
                let tile_idx = map.xy_idx(pos.x, pos.y) as i32;
                let target = if let Some(aoe) = ecs.read_storage::<AreaOfEffect>().get(spell_entity)
                {
                    Targets::Tiles {
                        tiles: aoe_tiles(&map, rltk::Point::new(pos.x, pos.y), aoe.radius),
                    }
                } else {
                    Targets::Tile { tile_idx }
                };
                add_effect(
                    None,
                    EffectType::SpellUse {
                        spell: find_spell_entity(ecs, &effect.spell).unwrap(),
                    },
                    target,
                );
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
