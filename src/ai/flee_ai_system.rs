use rltk::DijkstraMap;
use specs::{Entities, Join, System, WriteExpect, WriteStorage};

use crate::{spatial, ApplyMove, Map, MyTurn, Position, WantsToFlee};

pub struct FleeAI;

impl<'a> System<'a> for FleeAI {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, WantsToFlee>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut want_flee, mut positions, mut map, entities, mut apply_move) = data;

        let mut turn_done = Vec::new();
        for (entity, pos, flee, _myturn) in (&entities, &mut positions, &want_flee, &turns).join() {
            turn_done.push(entity);
            let my_idx = map.xy_idx(pos.x, pos.y);
            map.populate_blocked();
            let flee_map = DijkstraMap::new(
                map.width as usize,
                map.height as usize,
                &flee.indices,
                &*map,
                100.0,
            );
            if let Some(flee_tgt_idx) = DijkstraMap::find_highest_exit(&flee_map, my_idx, &*map) {
                if !spatial::is_blocked(flee_tgt_idx) {
                    apply_move
                        .insert(
                            entity,
                            ApplyMove {
                                dest_idx: flee_tgt_idx,
                            },
                        )
                        .expect("Unable to insert");
                    turn_done.push(entity);
                }
            }
        }

        want_flee.clear();

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
