use std::collections::HashMap;

use specs::{Entities, Entity, Join, ReadExpect, System, WriteStorage};

use crate::{spatial, Chasing, EntityMoved, Map, MyTurn, Position, Viewshed};

pub struct ChaseAI;

impl<'a> System<'a> for ChaseAI {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, Chasing>,
        WriteStorage<'a, Position>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, EntityMoved>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut chasing, mut positions, map, mut viewsheds, mut entity_moved, entities) =
            data;

        let mut targets: HashMap<Entity, (i32, i32)> = HashMap::new();
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
        for (entity, pos, _chase, viewshed, _myturn) in
            (&entities, &mut positions, &chasing, &mut viewsheds, &turns).join()
        {
            turn_done.push(entity);
            let (tgt_x, tgt_y) = targets[&entity];
            let idx = map.xy_idx(pos.x, pos.y);
            let path = rltk::a_star_search(idx, map.xy_idx(tgt_x, tgt_y), &*map);
            if path.success && path.steps.len() > 1 && path.steps.len() < 15 {
                let (x, y) = map.idx_xy(path.steps[1]);
                pos.x = x;
                pos.y = y;
                entity_moved
                    .insert(entity, EntityMoved {})
                    .expect("Unable to insert marker");
                spatial::move_entity(entity, idx, path.steps[1]);
                viewshed.dirty = true;
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
