use rltk::RandomNumberGenerator;

use crate::TileType;

use super::{
    area_starting_points::{AreaStartingPosition, XStart, YStart},
    cull_unreachable::CullUnreachable,
    distant_exit::DistantExit,
    drunkard::DrunkardsWalkBuilder,
    voronoi_spawning::VoronoiSpawning,
    BuilderChain, BuilderMap, MetaMapBuilder,
};

pub fn limestone_cavern_builder(
    new_depth: i32,
    _rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Limestone Caverns");
    chain.start_with(DrunkardsWalkBuilder::winding_passages());
    chain.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Center));
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain.with(CaveDecorator::new());
    chain
}

pub struct CaveDecorator {}

impl MetaMapBuilder for CaveDecorator {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CaveDecorator {
    pub fn new() -> Box<CaveDecorator> {
        Box::new(CaveDecorator {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let old_map = build_data.map.clone();
        for (idx, tt) in build_data.map.tiles.iter_mut().enumerate() {
            *tt = match *tt {
                TileType::Floor => {
                    // Gravel Spawning
                    if rng.roll_dice(1, 6) == 1 {
                        TileType::Gravel
                    // Spawn passable pools
                    } else if rng.roll_dice(1, 10) == 1 {
                        TileType::ShallowWater
                    } else {
                        *tt
                    }
                }
                TileType::Wall => {
                    // Spawn deep pools and stalactites
                    let (x, y) = old_map.idx_xy(idx);
                    // Count neighbors
                    let neighbor_walls = [
                        (x > 0, -1),
                        (x < build_data.width - 1, 1),
                        (y > 0, -(build_data.width as isize)),
                        (y < build_data.height - 1, build_data.width as isize),
                    ]
                    .iter()
                    .filter(|&&(in_bounds, offset)| {
                        in_bounds
                            && old_map.tiles[(idx as isize + offset) as usize] == TileType::Wall
                    })
                    .count();

                    match neighbor_walls {
                        2 => TileType::DeepWater,
                        1 => match rng.roll_dice(1, 4) {
                            1 => TileType::Stalactite,
                            2 => TileType::Stalagmite,
                            _ => *tt,
                        },
                        _ => *tt,
                    }
                }
                _ => *tt,
            };
        }
        build_data.take_snapshot();
        build_data.map.outdoors = false; // Underground
    }
}
