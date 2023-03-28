use super::{
    common::{paint, Symmetry},
    BuilderMap, InitialMapBuilder,
};
use crate::{Position, TileType};
use rltk::RandomNumberGenerator;

pub struct DrunkardsWalkBuilder {
    settings: DrunkardSettings,
}

impl InitialMapBuilder for DrunkardsWalkBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl DrunkardsWalkBuilder {
    pub fn open_area() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::StartingPoint,
                lifetime: 400,
                floor_ratio: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        })
    }

    pub fn open_halls() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                lifetime: 400,
                floor_ratio: 0.5,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        })
    }

    pub fn winding_passages() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                lifetime: 100,
                floor_ratio: 0.4,
                brush_size: 1,
                symmetry: Symmetry::None,
            },
        })
    }

    pub fn fat_passages() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                lifetime: 100,
                floor_ratio: 0.4,
                brush_size: 2,
                symmetry: Symmetry::None,
            },
        })
    }

    pub fn fearful_symmetry() -> Box<DrunkardsWalkBuilder> {
        Box::new(DrunkardsWalkBuilder {
            settings: DrunkardSettings {
                spawn_mode: DrunkSpawnMode::Random,
                lifetime: 100,
                floor_ratio: 0.4,
                brush_size: 1,
                symmetry: Symmetry::Both,
            },
        })
    }

    // Start at center -> Convert to floor tile
    // count floor space %, iterate till desired floor space %.
    // Spawn a drunkard at the starting point with "lifetime" and "position".
    // Decrement the drunkard's lifetime, have them move in random dir (4-sided), convert tile to floor.
    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // Set starting point; start at the middle
        let starting_position = Position {
            x: build_data.map.width / 2,
            y: build_data.map.height / 2,
        };

        let start_idx = build_data
            .map
            .xy_idx(starting_position.x, starting_position.y);
        build_data.map.tiles[start_idx] = TileType::Floor;

        let mut digger_count = 0;
        let total_tiles = build_data.map.width * build_data.map.height;
        let desired_floor_tiles = (self.settings.floor_ratio * total_tiles as f32) as usize;
        let mut floor_tile_count = build_data
            .map
            .tiles
            .iter()
            .filter(|a| **a == TileType::Floor)
            .count();

        while floor_tile_count < desired_floor_tiles {
            let mut did_something = false;
            let mut drunk_pos = match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => starting_position,
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        starting_position
                    } else {
                        Position {
                            x: rng.roll_dice(1, build_data.map.width - 3) + 1,
                            y: rng.roll_dice(1, build_data.map.height - 3) + 1,
                        }
                    }
                }
            };
            let mut drunk_life = self.settings.lifetime;

            while drunk_life > 0 {
                let drunk_idx = build_data.map.xy_idx(drunk_pos.x, drunk_pos.y);
                if build_data.map.tiles[drunk_idx] == TileType::Wall {
                    did_something = true;
                }
                // Set as digger tile
                paint(
                    &mut build_data.map,
                    self.settings.symmetry,
                    self.settings.brush_size,
                    drunk_pos.x,
                    drunk_pos.y,
                );
                build_data.map.tiles[drunk_idx] = TileType::DownStairs;

                match rng.roll_dice(1, 4) {
                    1 => {
                        if drunk_pos.x > 2 {
                            drunk_pos.x -= 1;
                        }
                    }
                    2 => {
                        if drunk_pos.x < build_data.map.width - 2 {
                            drunk_pos.x += 1;
                        }
                    }
                    3 => {
                        if drunk_pos.y > 2 {
                            drunk_pos.y -= 1;
                        }
                    }
                    _ => {
                        if drunk_pos.y < build_data.map.height - 2 {
                            drunk_pos.y += 1;
                        }
                    }
                }
                drunk_life -= 1;
            }
            if did_something {
                build_data.take_snapshot();
            }
            digger_count += 1;

            // Reset carved out tiles to floor type
            for t in build_data.map.tiles.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }

            floor_tile_count = build_data
                .map
                .tiles
                .iter()
                .filter(|tile| **tile == TileType::Floor)
                .count();
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode {
    StartingPoint,
    Random,
}

pub struct DrunkardSettings {
    pub spawn_mode: DrunkSpawnMode,
    pub lifetime: i32,
    pub floor_ratio: f32,
    pub symmetry: Symmetry,
    pub brush_size: i32,
}
