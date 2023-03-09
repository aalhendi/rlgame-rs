use super::{
    common::apply_horizontal_tunnel, common::apply_room_to_map, common::apply_vertical_tunnel, Map,
    MapBuilder,
};
use crate::{spawner, Position, Rect, TileType};
use rltk::RandomNumberGenerator;
use specs::World;

pub struct SimpleMapBuilder {
    map: Map,
    starting_position: Position,
    rooms: Vec<Rect>,
}

impl MapBuilder for SimpleMapBuilder {
    fn build_map(&mut self) {
        self.rooms_and_corridors();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for room in self.rooms.iter().skip(1) {
            spawner::spawn_room(ecs, room, self.map.depth);
        }
    }

    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }
}

impl SimpleMapBuilder {
    pub fn new(new_depth: i32) -> SimpleMapBuilder {
        SimpleMapBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            rooms: Vec::new(),
        }
    }

    /// Makes a new map using the algorithm from <http://rogueliketutorials.com/tutorials/tcod/part-3/>
    /// Returns map with random rooms and corridors to join them.
    pub fn rooms_and_corridors(&mut self) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, self.map.width - 1 - w) - 1;
            let y = rng.roll_dice(1, self.map.height - 1 - h) - 1;

            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in self.rooms.iter() {
                if new_room.intersects(other_room) {
                    ok = false;
                }
            }
            if ok {
                apply_room_to_map(&mut self.map, &new_room);

                if !self.rooms.is_empty() {
                    let new_center = new_room.center();
                    let old_center = self.rooms[self.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(
                            &mut self.map,
                            old_center.x,
                            new_center.x,
                            old_center.y,
                        );
                        apply_vertical_tunnel(
                            &mut self.map,
                            old_center.y,
                            new_center.y,
                            new_center.x,
                        );
                    } else {
                        apply_vertical_tunnel(
                            &mut self.map,
                            old_center.y,
                            new_center.y,
                            new_center.x,
                        );
                        apply_horizontal_tunnel(
                            &mut self.map,
                            old_center.x,
                            new_center.x,
                            old_center.y,
                        );
                    }
                }

                self.rooms.push(new_room)
            }
        }

        // Insert down stairs in center of last room
        let stairs_pos = self.rooms[self.rooms.len() - 1].center();
        let stairs_idx = self.map.xy_idx(stairs_pos.x, stairs_pos.y);
        self.map.tiles[stairs_idx] = TileType::DownStairs;

        self.starting_position = self.rooms[0].center();
    }
}
