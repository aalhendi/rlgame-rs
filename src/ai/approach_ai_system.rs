use specs::{Entities, Join, System, WriteExpect, WriteStorage};

use crate::{ApplyMove, Map, MyTurn, Position, WantsToApproach};

pub struct ApproachAI;

impl<'a> System<'a> for ApproachAI {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, WantsToApproach>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut want_approach, mut positions, map, entities, mut apply_move) = data;

        let mut turn_done = Vec::new();
        for (entity, pos, approach, _myturn) in
            (&entities, &mut positions, &want_approach, &turns).join()
        {
            turn_done.push(entity);
            let start_idx = map.xy_idx(pos.x, pos.y);
            let path = rltk::a_star_search(start_idx, approach.idx as usize, &*map);
            if path.success && path.steps.len() > 1 {
                apply_move
                    .insert(
                        entity,
                        ApplyMove {
                            dest_idx: path.steps[1],
                        },
                    )
                    .expect("Unable to insert");
            }
        }

        want_approach.clear();

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
