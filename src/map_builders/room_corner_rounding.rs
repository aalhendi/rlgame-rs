use super::{BuilderMap, MetaMapBuilder, Rect};
use crate::TileType;
use rltk::RandomNumberGenerator;

pub struct RoomCornerRounder {}

impl MetaMapBuilder for RoomCornerRounder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl RoomCornerRounder {
    pub fn new() -> Box<RoomCornerRounder> {
        Box::new(RoomCornerRounder {})
    }

    fn fill_if_corner(&mut self, build_data: &mut BuilderMap, x: i32, y: i32) {
        let w = build_data.map.width;
        let h = build_data.map.height;
        let idx = build_data.map.xy_idx(x, y);
        let tiles = &build_data.map.tiles;

        let neighbor_walls = {
            (x > 0 && tiles[idx - 1] == TileType::Wall) as i32
                + { y > 0 && tiles[idx - w as usize] == TileType::Wall } as i32
                + { x < w - 2 && tiles[idx + 1] == TileType::Wall } as i32
                + { y < h - 2 && tiles[idx + w as usize] == TileType::Wall } as i32
        };

        if neighbor_walls == 2 {
            build_data.map.tiles[idx] = TileType::Wall;
        }
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Room Rounding require a builder with room structures");
        }

        for room in rooms.iter() {
            self.fill_if_corner(build_data, room.x1 + 1, room.y1 + 1);
            self.fill_if_corner(build_data, room.x2, room.y1 + 1);
            self.fill_if_corner(build_data, room.x1 + 1, room.y2);
            self.fill_if_corner(build_data, room.x2, room.y2);

            build_data.take_snapshot();
        }
    }
}
