use std::collections::HashMap;

use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteStorage};

use crate::{ApplyMove, Chasing, Map, MyTurn, Position, TileSize};

pub struct ChaseAI;

impl<'a> System<'a> for ChaseAI {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, Chasing>,
        WriteStorage<'a, Position>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>,
        ReadStorage<'a, TileSize>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut chasing, mut positions, map, entities, mut apply_move, sizes) = data;

        let mut targets = HashMap::new();
        let mut end_chase = Vec::new();
        for (entity, _turn, chasing) in (&entities, &turns, &chasing).join() {
            if let Some(target_pos) = positions.get(chasing.target) {
                targets.insert(entity, (target_pos.x, target_pos.y));
            } else {
                end_chase.push(entity);
            }
        }

        for done in end_chase.iter() {
            chasing.remove(*done);
        }
        end_chase.clear();

        let mut turn_done = Vec::new();
        for (entity, pos, _chase, _myturn) in (&entities, &mut positions, &chasing, &turns).join() {
            turn_done.push(entity);
            let (tgt_x, tgt_y) = targets[&entity];
            let idx = map.xy_idx(pos.x, pos.y);
            let path = if let Some(size) = sizes.get(entity) {
                let mut map_copy = map.clone();
                map_copy.populate_blocked_multi(size.x, size.y);
                let start = map_copy.xy_idx(pos.x, pos.y) as i32;
                let end = map_copy.xy_idx(tgt_x, tgt_y) as i32;
                rltk::a_star_search(start, end, &map_copy)
            } else {
                rltk::a_star_search(idx, map.xy_idx(tgt_x, tgt_y), &*map)
            };
            if path.success && path.steps.len() > 1 && path.steps.len() < 15 {
                apply_move
                    .insert(
                        entity,
                        ApplyMove {
                            dest_idx: path.steps[1],
                        },
                    )
                    .expect("Unable to insert");
                turn_done.push(entity);
            } else {
                end_chase.push(entity);
            }
        }

        for done in end_chase.iter() {
            chasing.remove(*done);
        }
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
