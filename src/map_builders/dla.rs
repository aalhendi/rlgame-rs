use super::{
    common::{
        generate_voronoi_spawn_regions, paint, remove_unreachable_areas_get_most_distant, Symmetry,
    },
    Map, MapBuilder,
};
use crate::{spawner, Position, TileType, SHOW_MAPGEN_VISUALIZER};
use rltk::RandomNumberGenerator;
use specs::World;
use std::collections::HashMap;

#[derive(PartialEq, Copy, Clone, Default)]
pub enum DLAAlgorithm {
    #[default]
    WalkInwards,
    WalkOutwards,
    CentralAttractor,
}

/// <http://www.roguebasin.com/index.php?title=Diffusion-limited_aggregation>
#[derive(Default)]
pub struct DLABuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    algorithm: DLAAlgorithm,
    brush_size: i32,
    symmetry: Symmetry,
    floor_ratio: f32,
}

impl MapBuilder for DLABuilder {
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

impl DLABuilder {
    pub fn walk_inwards(new_depth: i32) -> DLABuilder {
        DLABuilder {
            map: Map::new(new_depth),
            depth: new_depth,
            algorithm: DLAAlgorithm::WalkInwards,
            brush_size: 1,
            symmetry: Symmetry::None,
            floor_ratio: 0.25,
            ..Default::default()
        }
    }

    pub fn walk_outwards(new_depth: i32) -> DLABuilder {
        DLABuilder {
            map: Map::new(new_depth),
            depth: new_depth,
            algorithm: DLAAlgorithm::WalkOutwards,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_ratio: 0.25,
            ..Default::default()
        }
    }

    pub fn central_attractor(new_depth: i32) -> DLABuilder {
        DLABuilder {
            map: Map::new(new_depth),
            depth: new_depth,
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::None,
            floor_ratio: 0.25,
            ..Default::default()
        }
    }

    pub fn insectoid(new_depth: i32) -> DLABuilder {
        DLABuilder {
            map: Map::new(new_depth),
            depth: new_depth,
            algorithm: DLAAlgorithm::CentralAttractor,
            brush_size: 2,
            symmetry: Symmetry::Horizontal,
            floor_ratio: 0.25,
            ..Default::default()
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        // Set starting point; start at the middle & carve seed
        self.starting_position = Position {
            x: self.map.width / 2,
            y: self.map.height / 2,
        };
        let start_idx = self
            .map
            .xy_idx(self.starting_position.x, self.starting_position.y);
        self.map.tiles[start_idx] = TileType::Floor;
        self.map.tiles[start_idx - 1] = TileType::Floor;
        self.map.tiles[start_idx + 1] = TileType::Floor;
        self.map.tiles[start_idx - self.map.width as usize] = TileType::Floor;
        self.map.tiles[start_idx + self.map.width as usize] = TileType::Floor;

        self.take_snapshot();

        // Random walker
        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.floor_ratio * total_tiles as f32) as usize;
        let mut floor_tile_count = self
            .map
            .tiles
            .iter()
            .filter(|tile| **tile == TileType::Floor)
            .count();
        while floor_tile_count < desired_floor_tiles {
            match self.algorithm {
                DLAAlgorithm::WalkInwards => {
                    let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = self.map.xy_idx(digger_x, digger_y);
                    while self.map.tiles[digger_idx] == TileType::Wall {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => {
                                if digger_x > 2 {
                                    digger_x -= 1;
                                }
                            }
                            2 => {
                                if digger_x < self.map.width - 2 {
                                    digger_x += 1;
                                }
                            }
                            3 => {
                                if digger_y > 2 {
                                    digger_y -= 1;
                                }
                            }
                            _ => {
                                if digger_y < self.map.height - 2 {
                                    digger_y += 1;
                                }
                            }
                        }
                        digger_idx = self.map.xy_idx(digger_x, digger_y);
                    }
                    paint(
                        &mut self.map,
                        self.symmetry,
                        self.brush_size,
                        prev_x,
                        prev_y,
                    );
                    self.take_snapshot();
                }
                DLAAlgorithm::WalkOutwards => {
                    let mut digger_x = self.starting_position.x;
                    let mut digger_y = self.starting_position.y;
                    let mut digger_idx = self.map.xy_idx(digger_x, digger_y);
                    while self.map.tiles[digger_idx] == TileType::Floor {
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => {
                                if digger_x > 2 {
                                    digger_x -= 1;
                                }
                            }
                            2 => {
                                if digger_x < self.map.width - 2 {
                                    digger_x += 1;
                                }
                            }
                            3 => {
                                if digger_y > 2 {
                                    digger_y -= 1;
                                }
                            }
                            _ => {
                                if digger_y < self.map.height - 2 {
                                    digger_y += 1;
                                }
                            }
                        }
                        digger_idx = self.map.xy_idx(digger_x, digger_y);
                    }
                    paint(
                        &mut self.map,
                        self.symmetry,
                        self.brush_size,
                        digger_x,
                        digger_y,
                    );
                }
                DLAAlgorithm::CentralAttractor => {
                    let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = self.map.xy_idx(digger_x, digger_y);

                    let mut path = rltk::line2d(
                        rltk::LineAlg::Bresenham,
                        rltk::Point::new(digger_x, digger_y),
                        rltk::Point::new(self.starting_position.x, self.starting_position.y),
                    );

                    while self.map.tiles[digger_idx] == TileType::Wall && !path.is_empty() {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        digger_x = path[0].x;
                        digger_y = path[0].y;
                        path.remove(0);
                        digger_idx = self.map.xy_idx(digger_x, digger_y);
                    }
                    paint(
                        &mut self.map,
                        self.symmetry,
                        self.brush_size,
                        prev_x,
                        prev_y,
                    );
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
    }
}
