use std::collections::HashMap;

use super::{
    HungerClock, HungerState, IsSerialized, Map, Name, Player, Position, Rect, Renderable,
    TileType, Viewshed,
};
use crate::{
    gamesystem::{attr_bonus, mana_at_level, player_hp_at_level},
    random_table::RandomTable,
    raws::{
        rawsmaster::{get_spawn_table_for_depth, spawn_named_entity, SpawnType},
        RAWS,
    },
    Attribute, Attributes, Pool, Pools, Skill, Skills,
};
use rltk::{RandomNumberGenerator, RGB};
use specs::{
    prelude::*,
    saveload::{MarkedBuilder, SimpleMarker},
};

const MAX_MONSTERS: i32 = 4;

/// Spawns the player and returns its entity object
pub fn player(ecs: &mut World, player_pos: Position) -> Entity {
    let mut skills = Skills::default();
    for skill in [Skill::Melee, Skill::Defense, Skill::Magic] {
        skills.skills.insert(skill, 1);
    }

    ecs.create_entity()
        .with(player_pos)
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(HungerClock {
            state: HungerState::WellFed,
            duration: 20,
        })
        .with(Attributes {
            might: Attribute {
                base: 11,
                modifiers: 0,
                bonus: attr_bonus(11),
            },
            fitness: Attribute {
                base: 11,
                modifiers: 0,
                bonus: attr_bonus(11),
            },
            quickness: Attribute {
                base: 11,
                modifiers: 0,
                bonus: attr_bonus(11),
            },
            intelligence: Attribute {
                base: 11,
                modifiers: 0,
                bonus: attr_bonus(11),
            },
        })
        .with(skills)
        .with(Pools {
            hit_points: Pool {
                current: player_hp_at_level(11, 1),
                max: player_hp_at_level(11, 1),
            },
            mana: Pool {
                current: mana_at_level(11, 1),
                max: mana_at_level(11, 1),
            },
            xp: 0,
            level: 1,
        })
        .marked::<SimpleMarker<IsSerialized>>()
        .build()
}

/// Calls spawn_region() with all possible_targets (floor tiles) from given room
pub fn spawn_room(
    map: &Map,
    rng: &mut RandomNumberGenerator,
    room: &Rect,
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let mut possible_targets: Vec<usize> = Vec::new();
    for y in room.y1 + 1..room.y2 {
        for x in room.x1 + 1..room.x2 {
            let idx = map.xy_idx(x, y);
            if map.tiles[idx] == TileType::Floor {
                possible_targets.push(idx);
            }
        }
    }

    spawn_region(map, rng, &possible_targets, map_depth, spawn_list);
}

pub fn spawn_region(
    // TODO: Remove?
    _map: &Map,
    rng: &mut RandomNumberGenerator,
    area: &[usize],
    map_depth: i32,
    spawn_list: &mut Vec<(usize, String)>,
) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();
    let mut areas: Vec<usize> = Vec::from(area);

    {
        let num_spawns = i32::min(
            areas.len() as i32,
            rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3,
        );
        if num_spawns == 0 {
            return;
        }

        for _ in 0..num_spawns {
            let array_index: usize = if areas.len() == 1 {
                0
            } else {
                (rng.roll_dice(1, areas.len() as i32) - 1) as usize
            };
            let map_idx = areas[array_index];
            spawn_points.insert(map_idx, spawn_table.roll(rng));
            areas.remove(array_index);
        }
    }

    // Spawning things
    for spawn in spawn_points.iter() {
        spawn_list.push((*spawn.0, spawn.1.to_string()));
    }
}

/// Spawns a named entity at the location map[idx]
pub fn spawn_entity(ecs: &mut World, (idx, name): &(&usize, &String)) {
    let map = ecs.fetch::<Map>();
    let (x, y) = map.idx_xy(**idx);
    std::mem::drop(map); // TODO: Needed?

    let spawn_result = spawn_named_entity(
        &RAWS.lock().unwrap(),
        ecs.create_entity(),
        name,
        SpawnType::AtPosition { x, y },
    );
    if spawn_result.is_some() {
        return;
    }

    rltk::console::log(format!("WARNING: Unable to spawn [{name}]!"));
}

fn room_table(map_depth: i32) -> RandomTable {
    get_spawn_table_for_depth(&RAWS.lock().unwrap(), map_depth)
}
