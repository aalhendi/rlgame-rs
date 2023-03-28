use crate::map::TileType;

use super::{BuilderMap, MetaMapBuilder};
use rltk::RandomNumberGenerator;

/// Searches a map via Dijkstra's, removes unreachable areas.
pub struct CullUnreachable {}

impl MetaMapBuilder for CullUnreachable {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CullUnreachable {
    pub fn new() -> Box<CullUnreachable> {
        Box::new(CullUnreachable {})
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_pos = build_data.starting_position.as_ref().unwrap();
        let start_idx = build_data.map.xy_idx(starting_pos.x, starting_pos.y);
        build_data.map.populate_blocked();
        let map_starts: Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(
            build_data.map.width as usize,
            build_data.map.height as usize,
            &map_starts,
            &build_data.map,
            1000.0,
        );
        for (i, tile) in build_data.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                // Unreachable tile -> Wall
                if distance_to_start == std::f32::MAX {
                    *tile = TileType::Wall;
                }
            }
        }
    }
}
