use std::sync::Mutex;

use specs::Entity;

use crate::{tile_walkable, Map, RunState};

struct SpatialMap {
    blocked: Vec<(bool, bool)>, // is_map_blocked, is_entity_blocked
    tile_content: Vec<Vec<(Entity, bool)>>,
}

impl SpatialMap {
    // deliberately not public
    // avoid sharing data directly, use API instead
    fn new() -> Self {
        Self {
            blocked: Vec::new(),
            tile_content: Vec::new(),
        }
    }
}

// allows access without burdening Specs' resources system
lazy_static! {
    static ref SPATIAL_MAP: Mutex<SpatialMap> = Mutex::new(SpatialMap::new());
}

pub fn populate_blocked_from_map(map: &Map) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    for (i, tile) in map.tiles.iter().enumerate() {
        lock.blocked[i].0 = !tile_walkable(*tile);
    }
}

pub fn clear() {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked.iter_mut().for_each(|b| {
        b.0 = false;
        b.1 = false;
    });
    for content in lock.tile_content.iter_mut() {
        content.clear();
    }
}

//  realloc might be bit inefficient - but not used often
pub fn set_size(map_tile_count: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked = vec![(false, false); map_tile_count];
    lock.tile_content = vec![Vec::new(); map_tile_count];
}

pub fn is_blocked(idx: usize) -> bool {
    let lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked[idx].0 || lock.blocked[idx].1
}

pub fn index_entity(entity: Entity, idx: usize, blocks_tile: bool) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.tile_content[idx].push((entity, blocks_tile));
    if blocks_tile {
        lock.blocked[idx].1 = true;
    }
}

// iterating tile content via closure.
// avoids returning lock & ensuring its freed, that leaks too much impl detail from API
pub fn for_each_tile_content<F>(idx: usize, mut f: F)
where
    F: FnMut(Entity),
{
    let lock = SPATIAL_MAP.lock().unwrap();
    for entity in lock.tile_content[idx].iter() {
        f(entity.0);
    }
}

pub fn for_each_tile_content_with_gamemode<F>(idx: usize, mut f: F) -> RunState
where
    F: FnMut(Entity) -> Option<RunState>,
{
    let lock = SPATIAL_MAP.lock().unwrap();
    for entity in lock.tile_content[idx].iter() {
        if let Some(rs) = f(entity.0) {
            return rs;
        }
    }

    RunState::AwaitingInput
}

pub fn move_entity(entity: Entity, moving_from: usize, moving_to: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    let mut entity_blocks = false;
    lock.tile_content[moving_from].retain(|&(e, blocks)| {
        let keep = e != entity;
        if !keep {
            entity_blocks = blocks;
        }
        keep
    });
    lock.tile_content[moving_to].push((entity, entity_blocks));

    // Recalculate entity blocks for both tiles
    let from_blocked = lock.tile_content[moving_from]
        .iter()
        .any(|(_, blocks)| *blocks);

    let to_blocked = lock.tile_content[moving_to]
        .iter()
        .any(|(_, blocks)| *blocks);

    lock.blocked[moving_from].1 = from_blocked;
    lock.blocked[moving_to].1 = to_blocked;
}

pub fn remove_entity(entity: Entity, idx: usize) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.tile_content[idx].retain(|(e, _)| *e != entity);
    let from_blocked = lock.tile_content[idx].iter().any(|&(_, blocks)| blocks);
    lock.blocked[idx].1 = from_blocked;
}

pub fn set_blocked(idx: usize, blocked: bool) {
    let mut lock = SPATIAL_MAP.lock().unwrap();
    lock.blocked[idx] = (lock.blocked[idx].0, blocked);
}
