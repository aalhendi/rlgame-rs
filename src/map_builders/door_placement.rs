use super::{BuilderMap, MetaMapBuilder};
use crate::TileType;
use rltk::RandomNumberGenerator;

pub struct DoorPlacement {}

impl MetaMapBuilder for DoorPlacement {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.doors(rng, build_data);
    }
}

impl DoorPlacement {
    pub fn new() -> Box<DoorPlacement> {
        Box::new(DoorPlacement {})
    }

    fn doors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // TODO: dont have more than 2 doors per hall (start, end)
        if let Some(halls) = &build_data.corridors.clone() {
            for hall in halls.iter() {
                if hall.len() > 2 && self.door_possible(build_data, hall[0]) {
                    // We aren't interested in tiny corridors
                    build_data.spawn_list.push((hall[0], "Door".to_string()));
                }
            }
        } else {
            // No corridors
            let tiles = &build_data.map.tiles;
            for (i, tile) in tiles.iter().enumerate() {
                if *tile == TileType::Floor
                    && self.door_possible(build_data, i)
                    && rng.roll_dice(1, 3) == 1
                {
                    build_data.spawn_list.push((i, "Door".to_string()));
                }
            }
        }
    }

    fn door_possible(&self, build_data: &BuilderMap, idx: usize) -> bool {
        let (x, y) = build_data.map.idx_xy(idx);
        let tiles = &build_data.map.tiles;
        let w = build_data.map.width;
        let h = build_data.map.height;

        for (spawn_idx, _spawn_name) in build_data.spawn_list.iter() {
            if spawn_idx == &idx {
                return false;
            }
        }

        // Check for east-west door possibility
        if tiles[idx] == TileType::Floor
            && (x > 1 && tiles[idx - 1] == TileType::Floor)
            && (x < w - 2 && tiles[idx + 1] == TileType::Floor)
            && (y > 1 && tiles[idx - w as usize] == TileType::Wall)
            && (y < h - 2 && tiles[idx + w as usize] == TileType::Wall)
        {
            return true;
        }

        // Check for north-south door possibility
        if tiles[idx] == TileType::Floor
            && (x > 1 && tiles[idx - 1] == TileType::Wall)
            && (x < w - 2 && tiles[idx + 1] == TileType::Wall)
            && (y > 1 && tiles[idx - w as usize] == TileType::Floor)
            && (y < h - 2 && tiles[idx + w as usize] == TileType::Floor)
        {
            return true;
        }

        false
    }
}
