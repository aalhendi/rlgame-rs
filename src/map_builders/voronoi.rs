use super::{BuilderMap, InitialMapBuilder};
use crate::TileType;
use rltk::RandomNumberGenerator;

#[derive(PartialEq, Copy, Clone, Default)]
pub enum DistanceAlgorithm {
    #[default]
    Pythagoras,
    Manhattan,
    Chebyshev,
}

#[derive(Default)]
pub struct VoronoiCellBuilder {
    n_seeds: usize,
    distance_algorithm: DistanceAlgorithm,
}

impl InitialMapBuilder for VoronoiCellBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl VoronoiCellBuilder {
    pub fn pythagoras() -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder {
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Pythagoras,
        })
    }

    pub fn manhattan() -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder {
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Manhattan,
        })
    }

    pub fn chebyshev() -> Box<VoronoiCellBuilder> {
        Box::new(VoronoiCellBuilder {
            n_seeds: 64,
            distance_algorithm: DistanceAlgorithm::Chebyshev,
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let mut voronoi_seeds: Vec<(usize, rltk::Point)> = Vec::new();

        while voronoi_seeds.len() < self.n_seeds {
            let vx = rng.roll_dice(1, build_data.map.width - 1);
            let vy = rng.roll_dice(1, build_data.map.height - 1);
            let vidx = build_data.map.xy_idx(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        // Each tile is given membership of the Voronoi group to whom's seed it is physically closest.
        let mut voronoi_distance = vec![(0, 0.0f32); self.n_seeds];
        let mut voronoi_membership: Vec<i32> =
            vec![0; build_data.map.width as usize * build_data.map.height as usize];
        for (idx, v_id) in voronoi_membership.iter_mut().enumerate() {
            let (x, y) = build_data.map.idx_xy(idx);

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
                a_distance.partial_cmp(b_distance).unwrap()
            });

            *v_id = voronoi_distance[0].0 as i32;
        }

        // Drawing the map
        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let mut neighbors = 0;
                let my_idx = build_data.map.xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];
                // Count how many neighboring tiles are in a different Voronoi group
                if voronoi_membership[build_data.map.xy_idx(x - 1, y)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[build_data.map.xy_idx(x + 1, y)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[build_data.map.xy_idx(x, y - 1)] != my_seed {
                    neighbors += 1;
                }
                if voronoi_membership[build_data.map.xy_idx(x, y + 1)] != my_seed {
                    neighbors += 1;
                }

                // 0: entirely in voronoi group: place a floor.
                // 1: only borders 1 other group - can also place a floor (to ensure we can walk around the map).
                if neighbors < 2 {
                    build_data.map.tiles[my_idx] = TileType::Floor;
                }
            }
            build_data.take_snapshot();
        }
    }
}
