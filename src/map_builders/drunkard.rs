use super::{
    common::{
        generate_voronoi_spawn_regions, paint, remove_unreachable_areas_get_most_distant, Symmetry,
    },
    Map, MapBuilder,
};
use crate::{spawner, Position, TileType, SHOW_MAPGEN_VISUALIZER};
use rltk::RandomNumberGenerator;
use std::collections::HashMap;

pub struct DrunkardsWalkBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    settings: DrunkardSettings,
    spawn_list: Vec<(usize, String)>,
}

impl MapBuilder for DrunkardsWalkBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {
        self.build();
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
    }
}

impl DrunkardsWalkBuilder {
    pub fn new(new_depth: i32, settings: DrunkardSettings) -> DrunkardsWalkBuilder {
        DrunkardsWalkBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
            settings,
            spawn_list: Vec::new(),
        }
    }

    pub fn open_area(new_depth: i32) -> DrunkardsWalkBuilder {
        let settings = DrunkardSettings {
            spawn_mode: DrunkSpawnMode::StartingPoint,
            lifetime: 400,
            floor_ratio: 0.5,
            symmetry: Symmetry::None,
            brush_size: 1,
        };
        DrunkardsWalkBuilder::new(new_depth, settings)
    }

    pub fn open_halls(new_depth: i32) -> DrunkardsWalkBuilder {
        let settings = DrunkardSettings {
            spawn_mode: DrunkSpawnMode::Random,
            lifetime: 400,
            floor_ratio: 0.5,
            symmetry: Symmetry::None,
            brush_size: 1,
        };
        DrunkardsWalkBuilder::new(new_depth, settings)
    }

    pub fn winding_passages(new_depth: i32) -> DrunkardsWalkBuilder {
        let settings = DrunkardSettings {
            spawn_mode: DrunkSpawnMode::Random,
            lifetime: 100,
            floor_ratio: 0.4,
            symmetry: Symmetry::None,
            brush_size: 1,
        };
        DrunkardsWalkBuilder::new(new_depth, settings)
    }

    pub fn fat_passages(new_depth: i32) -> DrunkardsWalkBuilder {
        let settings = DrunkardSettings {
            spawn_mode: DrunkSpawnMode::Random,
            lifetime: 100,
            floor_ratio: 0.4,
            brush_size: 2,
            symmetry: Symmetry::None,
        };
        DrunkardsWalkBuilder::new(new_depth, settings)
    }

    pub fn fearful_symmetry(new_depth: i32) -> DrunkardsWalkBuilder {
        let settings = DrunkardSettings {
            spawn_mode: DrunkSpawnMode::Random,
            lifetime: 100,
            floor_ratio: 0.4,
            brush_size: 1,
            symmetry: Symmetry::Both,
        };
        DrunkardsWalkBuilder::new(new_depth, settings)
    }

    // Start at center -> Convert to floor tile
    // count floor space %, iterate till desired floor space %.
    // Spawn a drunkard at the starting point with "lifetime" and "position".
    // Decrement the drunkard's lifetime, have them move in random dir (4-sided), convert tile to floor.
    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Set starting point; start at the middle
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        self.map.tiles[start_idx] = TileType::Floor;

        self.take_snapshot();

        let mut digger_count = 0;
        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.settings.floor_ratio * total_tiles as f32) as usize;
        let mut floor_tile_count = self
            .map
            .tiles
            .iter()
            .filter(|tile| **tile == TileType::Floor)
            .count();

        while floor_tile_count < desired_floor_tiles {
            let mut did_something = false;
            let mut drunk_pos = match self.settings.spawn_mode {
                DrunkSpawnMode::StartingPoint => self.starting_position.clone(),
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        self.starting_position.clone()
                    } else {
                        Position {
                            x: rng.roll_dice(1, self.map.width - 3) + 1,
                            y: rng.roll_dice(1, self.map.height - 3) + 1,
                        }
                    }
                }
            };
            let mut drunk_life = self.settings.lifetime;

            while drunk_life > 0 {
                let drunk_idx = self.map.xy_idx(drunk_pos.x, drunk_pos.y);
                if self.map.tiles[drunk_idx] == TileType::Wall {
                    did_something = true;
                }
                // Set as digger tile
                paint(
                    &mut self.map,
                    self.settings.symmetry,
                    self.settings.brush_size,
                    drunk_pos.x,
                    drunk_pos.y,
                );
                self.map.tiles[drunk_idx] = TileType::DownStairs;

                match rng.roll_dice(1, 4) {
                    1 => {
                        if drunk_pos.x > 2 {
                            drunk_pos.x -= 1;
                        }
                    }
                    2 => {
                        if drunk_pos.x < self.map.width - 2 {
                            drunk_pos.x += 1;
                        }
                    }
                    3 => {
                        if drunk_pos.y > 2 {
                            drunk_pos.y -= 1;
                        }
                    }
                    _ => {
                        if drunk_pos.y < self.map.height - 2 {
                            drunk_pos.y += 1;
                        }
                    }
                }
                drunk_life -= 1;
            }
            if did_something {
                self.take_snapshot();
            }
            digger_count += 1;

            // Reset carved out tiles to floor type
            for t in self.map.tiles.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }

            floor_tile_count = self
                .map
                .tiles
                .iter()
                .filter(|tile| **tile == TileType::Floor)
                .count();
        }

        let exit_tile_idx = remove_unreachable_areas_get_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        self.map.tiles[exit_tile_idx] = TileType::DownStairs;
        self.take_snapshot();

        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);

        // Spawn entities
        for area in self.noise_areas.iter() {
            spawner::spawn_region(
                &self.map,
                &mut rng,
                area.1,
                self.depth,
                &mut self.spawn_list,
            );
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
