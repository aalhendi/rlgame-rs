use specs::{Entities, Join, System, WriteExpect, WriteStorage};

use crate::{EntityMoved, Map, MyTurn, Position, Viewshed, WantsToApproach};

pub struct ApproachAI;

impl<'a> System<'a> for ApproachAI {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, WantsToApproach>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, EntityMoved>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut turns,
            mut want_approach,
            mut positions,
            mut map,
            mut viewsheds,
            mut entity_moved,
            entities,
        ) = data;

        let mut turn_done = Vec::new();
        for (entity, pos, approach, viewshed, _myturn) in (
            &entities,
            &mut positions,
            &want_approach,
            &mut viewsheds,
            &turns,
        )
            .join()
        {
            turn_done.push(entity);
            // TODO(aalhendi): Is this needed? For camera?
            let (approach_x, approach_y) = map.idx_xy(approach.idx as usize);
            let end_idx = map.xy_idx(approach_x, approach_y);
            let start_idx = map.xy_idx(pos.x, pos.y);
            let path = rltk::a_star_search(start_idx, end_idx, &*map);
            if path.success && path.steps.len() > 1 {
                let idx = map.xy_idx(pos.x, pos.y);
                map.blocked[idx] = false;
                let (x, y) = map.idx_xy(path.steps[1]);
                pos.x = x;
                pos.y = y;
                entity_moved
                    .insert(entity, EntityMoved {})
                    .expect("Unable to insert marker");
                map.blocked[path.steps[1]] = true;
                viewshed.dirty = true;
            }
        }

        want_approach.clear();

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
