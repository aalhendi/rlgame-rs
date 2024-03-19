use specs::{Entities, Join, ReadExpect, System, WriteExpect, WriteStorage};

use crate::{spatial, tile_walkable, ApplyMove, Map, MoveMode, Movement, MyTurn, Position};

pub struct DefaultMoveAI;

impl<'a> System<'a> for DefaultMoveAI {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, MoveMode>,
        WriteStorage<'a, Position>,
        ReadExpect<'a, Map>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        Entities<'a>,
        WriteStorage<'a, ApplyMove>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, mut move_mode, mut positions, map, mut rng, entities, mut apply_move) =
            data;

        let mut turn_done = Vec::new();
        for (entity, pos, mode, _myturn) in
            (&entities, &mut positions, &mut move_mode, &turns).join()
        {
            turn_done.push(entity);

            match &mut mode.mode {
                Movement::Static => {}
                Movement::Random => {
                    let mut x = pos.x;
                    let mut y = pos.y;
                    let move_roll = rng.roll_dice(1, 5);
                    match move_roll {
                        1 => x -= 1,
                        2 => x += 1,
                        3 => y -= 1,
                        4 => y += 1,
                        _ => {}
                    }

                    // Check bounds
                    // TODO: Abstract bounds checking to method. Used in multiple places.
                    if x > 0 && x < map.width - 1 && y > 0 && y < map.height - 1 {
                        let dest_idx = map.xy_idx(x, y);
                        if !spatial::is_blocked(dest_idx) {
                            apply_move
                                .insert(entity, ApplyMove { dest_idx })
                                .expect("Unable to insert");
                            turn_done.push(entity);
                        }
                    }
                }
                Movement::RandomWaypoint { path } => {
                    match path {
                        Some(path) => {
                            // We have a target - go there
                            if path.len() > 1 {
                                if !spatial::is_blocked(path[1]) {
                                    apply_move
                                        .insert(entity, ApplyMove { dest_idx: path[1] })
                                        .expect("Unable to insert");
                                    path.remove(0); // Remove the first step in the path
                                    turn_done.push(entity);
                                }
                                // Otherwise we wait a turn to see if the path clears up
                            } else {
                                mode.mode = Movement::RandomWaypoint { path: None };
                            }
                        }
                        None => {
                            let target_x = rng.roll_dice(1, map.width - 2);
                            let target_y = rng.roll_dice(1, map.height - 2);
                            let tgt_idx = map.xy_idx(target_x, target_y);
                            if tile_walkable(map.tiles[tgt_idx]) {
                                let path =
                                    rltk::a_star_search(map.xy_idx(pos.x, pos.y), tgt_idx, &*map);
                                if path.success && path.steps.len() > 1 {
                                    mode.mode = Movement::RandomWaypoint {
                                        path: Some(path.steps),
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}
