use rltk::DijkstraMap;
use specs::{Entities, Join, System, WriteExpect, WriteStorage};

use crate::{EntityMoved, Map, MyTurn, Position, Viewshed, WantsToFlee};

pub struct FleeAI;

impl<'a> System<'a> for FleeAI {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, WantsToFlee>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Map>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, EntityMoved>,
        Entities<'a>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut turns,
            mut want_flee,
            mut positions,
            mut map,
            mut viewsheds,
            mut entity_moved,
            entities,
        ) = data;

        let mut turn_done = Vec::new();
        for (entity, pos, flee, viewshed, _myturn) in (
            &entities,
            &mut positions,
            &want_flee,
            &mut viewsheds,
            &turns,
        )
            .join()
        {
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
                if !map.blocked[flee_tgt_idx] {
                    map.blocked[my_idx] = false;
                    map.blocked[flee_tgt_idx] = true;
                    viewshed.dirty = true;
                    let (x, y) = map.idx_xy(flee_tgt_idx);
                    pos.x = x;
                    pos.y = y;
                    entity_moved
                        .insert(entity, EntityMoved {})
                        .expect("Unable to insert marker");
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
