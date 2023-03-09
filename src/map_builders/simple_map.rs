use super::{
    common::apply_horizontal_tunnel, common::apply_room_to_map, common::apply_vertical_tunnel, Map,
    MapBuilder,
};
use crate::{Rect, TileType};
use rltk::RandomNumberGenerator;
//use specs::prelude::*;

pub struct SimpleMapBuilder {}

impl MapBuilder for SimpleMapBuilder {
    fn build(new_depth: i32) -> Map {
        let mut map = Map::new(new_depth);
        SimpleMapBuilder::rooms_and_corridors(&mut map);
        map
    }
}

impl SimpleMapBuilder {
    /// Makes a new map using the algorithm from <http://rogueliketutorials.com/tutorials/tcod/part-3/>
    /// Returns map with random rooms and corridors to join them.
    pub fn rooms_and_corridors(map: &mut Map) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - 1 - w) - 1;
            let y = rng.roll_dice(1, map.height - 1 - h) - 1;

            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersects(other_room) {
                    ok = false;
                }
            }
            if ok {
                apply_room_to_map(map, &new_room);

                if !map.rooms.is_empty() {
                    let new_center = new_room.center();
                    let old_center = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(map, old_center.x, new_center.x, old_center.y);
                        apply_vertical_tunnel(map, old_center.y, new_center.y, new_center.x);
                    } else {
                        apply_vertical_tunnel(map, old_center.y, new_center.y, new_center.x);
                        apply_horizontal_tunnel(map, old_center.x, new_center.x, old_center.y);
                    }
                }

                map.rooms.push(new_room)
            }
        }

        // Insert down stairs in center of last room
        let stairs_pos = map.rooms[map.rooms.len() - 1].center();
        let stairs_idx = map.xy_idx(stairs_pos.x, stairs_pos.y);
        map.tiles[stairs_idx] = TileType::DownStairs;
    }
}
