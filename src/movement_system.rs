use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    spatial, ApplyMove, ApplyTeleport, BlocksTile, EntityMoved, Map, OtherLevelPosition, Position,
    RunState, Viewshed,
};

pub struct MovementSystem;

impl<'a> System<'a> for MovementSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, BlocksTile>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>,
        WriteStorage<'a, ApplyTeleport>,
        WriteStorage<'a, OtherLevelPosition>,
        WriteStorage<'a, EntityMoved>,
        WriteStorage<'a, Viewshed>,
        ReadExpect<'a, Entity>,
        WriteExpect<'a, RunState>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut position,
            _blockers,
            entities,
            mut apply_move,
            mut apply_teleport,
            mut other_level,
            mut moved,
            mut viewsheds,
            player_entity,
            mut runstate,
        ) = data;

        // Apply teleports
        for (entity, teleport) in (&entities, &apply_teleport).join() {
            let dest_idx = map.xy_idx(teleport.dest_x, teleport.dest_y);
            // Current floor
            if teleport.dest_depth == map.depth {
                apply_move
                    .insert(entity, ApplyMove { dest_idx })
                    .expect("Unable to insert");
            } else if entity == *player_entity {
                *runstate = RunState::TeleportingToOtherLevel {
                    x: teleport.dest_x,
                    y: teleport.dest_y,
                    depth: teleport.dest_depth,
                };
            } else if let Some(pos) = position.get(entity) {
                let idx = map.xy_idx(pos.x, pos.y);
                spatial::remove_entity(entity, idx);
                other_level
                    .insert(
                        entity,
                        OtherLevelPosition {
                            x: teleport.dest_x,
                            y: teleport.dest_y,
                            depth: teleport.dest_depth,
                        },
                    )
                    .expect("Unable to insert");
                position.remove(entity);
            }
        }
        apply_teleport.clear();

        // Apply broad movement
        for (entity, movement, pos) in (&entities, &apply_move, &mut position).join() {
            let start_idx = map.xy_idx(pos.x, pos.y);
            spatial::move_entity(entity, start_idx, movement.dest_idx);
            let (x, y) = map.idx_xy(movement.dest_idx);
            pos.x = x;
            pos.y = y;
            if let Some(vs) = viewsheds.get_mut(entity) {
                vs.dirty = true;
            }
            moved
                .insert(entity, EntityMoved {})
                .expect("Unable to insert");
        }
        apply_move.clear();
    }
}
