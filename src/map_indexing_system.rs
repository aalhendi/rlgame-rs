use super::{BlocksTile, Map, Position};
use specs::prelude::*;

pub struct MapIndexingSystem;

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, positions, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();
        for (position, entity) in (&positions, &entities).join() {
            let idx = map.xy_idx(position.x, position.y);

            // Update blocked_tiles if theres a blocking entity
            map.blocked[idx] = blockers.get(entity).is_some();

            // Push the entity to appropriate index slot. Its a copy type (we dont want to move in or the ECS will lose it).
            map.tile_content[idx].push(entity);
        }
    }
}
