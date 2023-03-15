use super::{
    common::{generate_voronoi_spawn_regions, remove_unreachable_areas_get_most_distant},
    Map, MapBuilder,
};
use crate::{spawner, Position, TileType, SHOW_MAPGEN_VISUALIZER};
use rltk::RandomNumberGenerator;
use specs::World;
use std::collections::HashMap;

#[derive(PartialEq, Copy, Clone, Default)]
pub enum DistanceAlgorithm {
    #[default]
    Pythagoras,
    Manhattan,
    Chebyshev,
}

#[derive(Default)]
pub struct VoronoiCellBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    noise_areas: HashMap<i32, Vec<usize>>,
    n_seeds: usize,
    distance_algorithm: DistanceAlgorithm,
}

impl MapBuilder for VoronoiCellBuilder {
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

impl VoronoiCellBuilder {
    pub fn pythagoras(new_depth: i32) -> VoronoiCellBuilder {
        VoronoiCellBuilder {
            map: Map::new(new_depth),
            depth: new_depth,
            distance_algorithm: DistanceAlgorithm::Pythagoras,
            n_seeds: 64,
            ..Default::default()
        }
    }

    pub fn manhattan(new_depth: i32) -> VoronoiCellBuilder {
        VoronoiCellBuilder {
            map: Map::new(new_depth),
            depth: new_depth,
            distance_algorithm: DistanceAlgorithm::Manhattan,
            n_seeds: 64,
            ..Default::default()
        }
    }

    pub fn chebyshev(new_depth: i32) -> VoronoiCellBuilder {
        VoronoiCellBuilder {
            map: Map::new(new_depth),
            depth: new_depth,
            distance_algorithm: DistanceAlgorithm::Chebyshev,
            n_seeds: 64,
            ..Default::default()
        }
    }

    fn build(&mut self) {
        let mut rng = RandomNumberGenerator::new();

        let mut voronoi_seeds: Vec<(usize, rltk::Point)> = Vec::new();

        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng.roll_dice(1, self.map.width - 1);
            let vy = rng.roll_dice(1, self.map.height - 1);
            let vidx = self.map.xy_idx(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        // Each tile is given membership of the Voronoi group to whom's seed it is physically closest.
        let mut voronoi_distance = vec![(0, 0.0f32); self.n_seeds];
        let mut voronoi_membership: Vec<i32> =
            vec![0; self.map.width as usize * self.map.height as usize];
        for (idx, v_id) in voronoi_membership.iter_mut().enumerate() {
            let x = idx as i32 % self.map.width;
            let y = idx as i32 / self.map.width;

            for (seed, (_map_idx, pos)) in voronoi_seeds.iter().enumerate() {
                let distance = match self.distance_algorithm {
                    DistanceAlgorithm::Pythagoras => rltk::DistanceAlg::PythagorasSquared
                        .distance2d(rltk::Point::new(x, y), *pos),
                    DistanceAlgorithm::Manhattan => {
                        rltk::DistanceAlg::Manhattan.distance2d(rltk::Point::new(x, y), *pos)
                    }
                    DistanceAlgorithm::Chebyshev => {
                        rltk::DistanceAlg::Chebyshev.distance2d(rltk::Point::new(x, y), *pos)
                    }
                };
                voronoi_distance[seed] = (seed, distance);
            }

            voronoi_distance.sort_by(|(_a_seed, a_distance), (_b_seed, b_distance)| {
                a_distance.partial_cmp(&b_distance).unwrap()
            });

            *v_id = voronoi_distance[0].0 as i32;
        }

        // Drawing the map
        for y in 1..self.map.height - 1 {
            for x in 1..self.map.width - 1 {
                let mut neighbors = 0;
                let my_idx = self.map.xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];
                // Count how many neighboring tiles are in a different Voronoi group
                if voronoi_membership[self.map.xy_idx(x - 1, y)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[self.map.xy_idx(x + 1, y)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[self.map.xy_idx(x, y - 1)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[self.map.xy_idx(x, y + 1)] != my_seed {
                    neighbors += 1;
                }

                // 0: entirely in voronoi group: place a floor.
                // 1: only borders 1 other group - can also place a floor (to ensure we can walk around the map).
                if neighbors < 2 {
                    self.map.tiles[my_idx] = TileType::Floor;
                }
            }
            self.take_snapshot();
        }

        // Set starting position
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
        self.take_snapshot();

        let exit_tile_idx = remove_unreachable_areas_get_most_distant(&mut self.map, start_idx);
        self.take_snapshot();

        // Set exit tile
        self.map.tiles[exit_tile_idx] = TileType::DownStairs;
        self.take_snapshot();

        // Generate noise map for entity spawning
        self.noise_areas = generate_voronoi_spawn_regions(&self.map, &mut rng);
    }
}
