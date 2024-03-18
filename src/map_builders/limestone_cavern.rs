use rltk::RandomNumberGenerator;

use crate::TileType;

use super::{
    area_ending_point::{AreaEndingPosition, XEnd, YEnd},
    area_starting_points::{AreaStartingPosition, XStart, YStart},
    bsp_dungeon::BspDungeonBuilder,
    cellular_automata::CellularAutomataBuilder,
    cull_unreachable::CullUnreachable,
    distant_exit::DistantExit,
    dla::DLABuilder,
    drunkard::DrunkardsWalkBuilder,
    prefab_builder::{prefab_sections, PrefabBuilder},
    room_based_spawner::RoomBasedSpawner,
    room_draw::RoomDrawer,
    room_exploder::RoomExploder,
    room_sorter::{RoomSort, RoomSorter},
    rooms_corridors_nearest::NearestCorridors,
    voronoi_spawning::VoronoiSpawning,
    BuilderChain, BuilderMap, MetaMapBuilder,
};

pub fn limestone_cavern_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
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

pub fn limestone_deep_cavern_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Deep Limestone Caverns");
    chain.start_with(DLABuilder::central_attractor());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Top));
    chain.with(VoronoiSpawning::new());
    chain.with(DistantExit::new());
    chain.with(CaveDecorator::new());
    chain.with(PrefabBuilder::sectional(prefab_sections::ORC_CAMP));
    chain
}

pub fn limestone_transition_builder(
    new_depth: i32,
    _rng: &mut RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Dwarf Fort - Upper Reaches");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Center));
    chain.with(VoronoiSpawning::new());
    chain.with(CaveDecorator::new());
    chain.with(CaveTransition::new());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Center));
    chain.with(CullUnreachable::new());
    // Force Exit to be on right side of map
    chain.with(AreaEndingPosition::new(XEnd::Right, YEnd::Center));
    chain
}

pub struct CaveDecorator {}

impl MetaMapBuilder for CaveDecorator {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
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

pub struct CaveTransition {}

impl MetaMapBuilder for CaveTransition {
    fn build_map(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CaveTransition {
    pub fn new() -> Box<CaveTransition> {
        Box::new(CaveTransition {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        build_data.map.depth = 5;
        build_data.take_snapshot();

        // Build a BSP-based dungeon
        let mut builder = BuilderChain::new(5, build_data.width, build_data.height, "New Map");
        builder.start_with(BspDungeonBuilder::new());
        builder.with(RoomDrawer::new());
        builder.with(RoomSorter::new(RoomSort::Rightmost));
        builder.with(NearestCorridors::new());
        builder.with(RoomExploder::new());
        builder.with(RoomBasedSpawner::new());
        builder.build_map(rng);

        // Add the history to our history
        build_data.history.extend(builder.build_data.history);
        build_data.take_snapshot();

        // Copy the right half of the BSP map into our map
        for x in build_data.map.width / 2..build_data.map.width {
            for y in 0..build_data.map.height {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = builder.build_data.map.tiles[idx];
            }
        }
        build_data.take_snapshot();

        // Keep Voronoi spawn data from the left half of the map
        let w = build_data.map.width;
        build_data.spawn_list.retain(|(s_idx, _s_name)| {
            let x = *s_idx as i32 / w;
            x < w / 2
        });

        // Keep room spawn data from the right half of the map
        for s in builder.build_data.spawn_list.iter() {
            let x = s.0 as i32 / w;
            if x > w / 2 {
                build_data.spawn_list.push(s.clone());
            }
        }
    }
}
