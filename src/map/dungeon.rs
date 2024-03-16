use crate::{map_builders::level_builder, OtherLevelPosition, Position, TileType, Viewshed};

use super::Map;
use rltk::Point;
use serde::{Deserialize, Serialize};
use specs::{Entity, Join, World, WorldExt};
use std::collections::HashMap;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct MasterDungeonMap {
    maps: HashMap<i32, Map>,
}

impl MasterDungeonMap {
    pub fn new() -> MasterDungeonMap {
        MasterDungeonMap {
            maps: HashMap::new(),
        }
    }

    pub fn store_map(&mut self, map: &Map) {
        self.maps.insert(map.depth, map.clone());
    }

    pub fn get_map(&self, depth: i32) -> Option<Map> {
        self.maps.get(&depth).cloned()
    }
}

pub fn level_transition(ecs: &mut World, new_depth: i32, offset: i32) -> Option<Vec<Map>> {
    // Obtain the master dungeon map
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();

    // Do we already have a map?
    let is_map_exists = dungeon_master.get_map(new_depth).is_some();
    std::mem::drop(dungeon_master);
    if is_map_exists {
        transition_to_existing_map(ecs, new_depth, offset);
        None
    } else {
        Some(transition_to_new_map(ecs, new_depth))
    }
}

fn transition_to_new_map(ecs: &mut World, new_depth: i32) -> Vec<Map> {
    let mut rng = ecs.write_resource::<rltk::RandomNumberGenerator>();
    let mut builder = level_builder(new_depth, &mut rng, 80, 50);
    builder.build_map(&mut rng);

    // Set upstairs
    if new_depth > 1 {
        if let Some(pos) = &builder.build_data.starting_position {
            let up_idx = builder.build_data.map.xy_idx(pos.x, pos.y);
            builder.build_data.map.tiles[up_idx] = TileType::UpStairs;
        }
    }
    let mapgen_history = builder.build_data.history.clone();
    let player_start = {
        let mut worldmap_resource = ecs.write_resource::<Map>();
        *worldmap_resource = builder.build_data.map.clone();
        *builder.build_data.starting_position.as_mut().unwrap()
    };

    // Spawn bad guys
    std::mem::drop(rng);
    builder.spawn_entities(ecs);

    // Place the player and update resources
    let mut player_position = ecs.write_resource::<Point>();
    *player_position = Point::new(player_start.x, player_start.y);
    let mut position_components = ecs.write_storage::<Position>();
    let player_entity = ecs.fetch::<Entity>();
    if let Some(player_pos_comp) = position_components.get_mut(*player_entity) {
        *player_pos_comp = player_start;
    }

    // Mark the player's visibility as dirty
    let mut viewshed_components = ecs.write_storage::<Viewshed>();
    if let Some(player_vs) = viewshed_components.get_mut(*player_entity) {
        player_vs.dirty = true;
    }

    // Store the newly minted map
    let mut dungeon_master = ecs.write_resource::<MasterDungeonMap>();
    dungeon_master.store_map(&builder.build_data.map);

    mapgen_history
}

fn transition_to_existing_map(ecs: &mut World, new_depth: i32, offset: i32) {
    let dungeon_master = ecs.read_resource::<MasterDungeonMap>();
    let map = dungeon_master.get_map(new_depth).unwrap();
    let mut worldmap_resource = ecs.write_resource::<Map>();
    let player_entity = ecs.fetch::<Entity>();

    // Find the down stairs and place the player
    let mut player_position = ecs.write_resource::<Point>();
    let mut position_components = ecs.write_storage::<Position>();
    let stair_type = if offset < 0 {
        TileType::DownStairs
    } else {
        TileType::UpStairs
    };
    for (idx, tt) in map.tiles.iter().enumerate() {
        if *tt != stair_type {
            continue;
        }

        let (x, y) = map.idx_xy(idx);
        *player_position = Point::new(x, y);
        if let Some(player_pos_comp) = position_components.get_mut(*player_entity) {
            player_pos_comp.x = x;
            player_pos_comp.y = y;
        }
    }

    // Replace map
    *worldmap_resource = map;

    // Mark the player's visibility as dirty
    let mut viewshed_components = ecs.write_storage::<Viewshed>();
    if let Some(player_vs) = viewshed_components.get_mut(*player_entity) {
        player_vs.dirty = true;
    }
}

pub fn freeze_level_entities(ecs: &mut World) {
    // Obtain ECS access
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_level_positions = ecs.write_storage::<OtherLevelPosition>();
    let player_entity = ecs.fetch::<Entity>();
    let depth = ecs.fetch::<Map>().depth;

    // Create OtherLevelPosition
    let mut pos_to_delete: Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &positions).join() {
        // Don't delete the player
        if entity == *player_entity {
            continue;
        }
        other_level_positions
            .insert(
                entity,
                OtherLevelPosition {
                    x: pos.x,
                    y: pos.y,
                    depth,
                },
            )
            .expect("Other level position insert fail");

        pos_to_delete.push(entity);
    }

    // Remove positions
    for p in pos_to_delete.iter() {
        positions.remove(*p);
    }
}

pub fn thaw_level_entities(ecs: &mut World) {
    // Obtain ECS access
    let entities = ecs.entities();
    let mut positions = ecs.write_storage::<Position>();
    let mut other_level_positions = ecs.write_storage::<OtherLevelPosition>();
    let player_entity = ecs.fetch::<Entity>();
    let depth = ecs.fetch::<Map>().depth;

    // Find OtherLevelPosition
    let mut pos_to_delete: Vec<Entity> = Vec::new();
    for (entity, pos) in (&entities, &other_level_positions).join() {
        // Dont restore entities on other depths. Dont resore player
        if pos.depth != depth || entity == *player_entity {
            continue;
        }

        positions
            .insert(entity, Position { x: pos.x, y: pos.y })
            .expect("Position insert fail");
        pos_to_delete.push(entity);
    }

    // Remove other level positions
    for p in pos_to_delete.iter() {
        other_level_positions.remove(*p);
    }
}
