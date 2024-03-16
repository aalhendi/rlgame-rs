use crate::{spatial, Pools};

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
    );

    fn run(&mut self, data: Self::SystemData) {
        let (map, positions, blockers, pools, entities) = data;

        spatial::clear();
        spatial::populate_blocked_from_map(&map);
        for (position, entity) in (&positions, &entities).join() {
            let alive = pools
                .get(entity)
                .map_or(true, |pools| pools.hit_points.current >= 1);

            if alive {
                let idx = map.xy_idx(position.x, position.y);
                spatial::index_entity(entity, idx, blockers.get(entity).is_some());
            }
        }
    }
}
