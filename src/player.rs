use super::{Map, Player, Position, RunState, State, Viewshed};
use crate::WINDOW_HEIGHT;
use crate::WINDOW_WIDTH;
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut ppos = ecs.write_resource::<Point>();
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let map = ecs.fetch::<Map>();

    for (_player, pos, viewshed) in (&mut players, &mut positions, &mut viewsheds).join() {
        let dest_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);
        if !map.blocked[dest_idx] {
            pos.x = (pos.x + delta_x).clamp(0, WINDOW_WIDTH - 1);
            pos.y = (pos.y + delta_y).clamp(0, WINDOW_HEIGHT - 1);
            ppos.x = pos.x;
            ppos.y = pos.y;

            viewshed.dirty = true;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    use VirtualKeyCode::*;
    match ctx.key {
        None => return RunState::Paused,
        Some(key) => match key {
            //Cardinal
            Left | Numpad4 | H => try_move_player(-1, 0, &mut gs.ecs),
            Right | Numpad6 | L => try_move_player(1, 0, &mut gs.ecs),
            Up | Numpad8 | K => try_move_player(0, -1, &mut gs.ecs),
            Down | Numpad2 | J => try_move_player(0, 1, &mut gs.ecs),

            //Diagonal
            Numpad1 | Y => try_move_player(-1, -1, &mut gs.ecs),
            Numpad9 | N => try_move_player(1, 1, &mut gs.ecs),
            Numpad7 | B => try_move_player(-1, 1, &mut gs.ecs),
            Numpad3 | U => try_move_player(1, -1, &mut gs.ecs),

            _ => return RunState::Paused,
        },
    }
    RunState::Running
}
