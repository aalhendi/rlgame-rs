use super::{Map, MapBuilder};
use crate::{spawner, Position, TileType, SHOW_MAPGEN_VISUALIZER};
use rltk::RandomNumberGenerator;
use specs::World;
use std::collections::HashMap;

/// Inspired by: <http://www.roguebasin.com/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels>
pub struct CellularAutomataBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
}

impl MapBuilder for CellularAutomataBuilder {
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

    fn spawn_entities(&mut self, ecs: &mut World) {
        for (_area_id, tile_ids) in self.noise_areas.iter() {
            spawner::spawn_region(ecs, tile_ids, self.depth);
        }
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
}

impl CellularAutomataBuilder {
    pub fn new(new_depth: i32) -> CellularAutomataBuilder {
        CellularAutomataBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            noise_areas: HashMap::new(),
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Create random map, 55% floor. Cellular automata are designed to make a level out of noise
        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = self.map.xy_idx(x, y);
                self.map.tiles[idx] = if roll > 55 {
                    TileType::Floor
                } else {
                    TileType::Wall
                }
            }
            self.take_snapshot();
        }

        // Now we iteratively apply cellular automata rules:
        // iterating each cell,
        // counting the number of neighbors,
        // and turning walls into floors or walls based on density.
        let map_width = self.map.width as usize;
        for _ in 0..15 {
            // Used to not write on the tiles we are counting, which gives a very odd map...
            let mut newtiles = self.map.tiles.clone();

            for y in 1..self.map.height - 1 {
                for x in 1..self.map.width - 1 {
                    let idx = self.map.xy_idx(x, y);
                    let mut neighbors = 0;
                    if self.map.tiles[idx - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - map_width] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + map_width] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - map_width - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx - map_width + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + map_width - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if self.map.tiles[idx + map_width + 1] == TileType::Wall {
                        neighbors += 1;
                    }

                    if neighbors > 4 || neighbors == 0 {
                        newtiles[idx] = TileType::Wall;
                    } else {
                        newtiles[idx] = TileType::Floor;
                    }
                }
            }

            self.map.tiles = newtiles.clone();
            self.take_snapshot();
        }

        // Find a starting point; start at the middle and walk left until we find an open tile
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let mut start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        while self.map.tiles[start_idx] != TileType::Floor {
            self.starting_position.x -= 1;
            start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y);
        }

        // Find all tiles we can reach from the starting point
        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(
            self.map.width,
            self.map.height,
            &map_starts,
            &self.map,
            200.0,
        );
        let (mut exit_tile_idx, mut exit_tile_distance) = (0, 0.0f32);
        for (i, tile) in self.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                // Unreachable tile -> Wall
                if distance_to_start == std::f32::MAX {
                    *tile = TileType::Wall;
                } else if distance_to_start > exit_tile_distance {
                    // Move exist if further than current exit
                    exit_tile_idx = i;
                    exit_tile_distance = distance_to_start;
                }
            }
        }
        self.take_snapshot();

        self.map.tiles[exit_tile_idx] = TileType::DownStairs;
        self.take_snapshot();

        // Build noise map for spawning entities later <https://thebookofshaders.com/12/>
        let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(rltk::NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let idx = self.map.xy_idx(x, y);
                if self.map.tiles[idx] == TileType::Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;

                    self.noise_areas
                        .entry(cell_value_f as i32)
                        .or_default()
                        .push(idx);
                }
            }
        }
    }
}
