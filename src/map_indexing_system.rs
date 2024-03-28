use crate::{spatial, Pools, TileSize};

use super::{BlocksTile, Map, Position};
use specs::prelude::*;

pub struct MapIndexingSystem;

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        ReadStorage<'a, Pools>,
        Entities<'a>,
        ReadStorage<'a, TileSize>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, positions, blockers, pools, entities, sizes) = data;

        spatial::clear();
        spatial::populate_blocked_from_map(&map);
        for (position, entity) in (&positions, &entities).join() {
            let alive = pools
                .get(entity)
                .map_or(true, |pools| pools.hit_points.current >= 1);

            if alive {
                let blocks_tile = blockers.get(entity).is_some();

                if let Some(size) = sizes.get(entity) {
                    // Multi-tile
                    for y in position.y..position.y + size.y {
                        for x in position.x..position.x + size.x {
                            if x > 0 && x < map.width - 1 && y > 0 && y < map.height - 1 {
                                let idx = map.xy_idx(x, y);
                                spatial::index_entity(entity, idx, blocks_tile);
                            }
                        }
                    }
                    continue;
                }
                // Single tile
                let idx = map.xy_idx(position.x, position.y);
                spatial::index_entity(entity, idx, blocks_tile);
            }
        }
    }
}
