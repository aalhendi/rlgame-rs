use super::{xy_idx, Player, Position, State, TileType};
use crate::WINDOW_HEIGHT;
use crate::WINDOW_WIDTH;
use rltk::{Rltk, VirtualKeyCode};
use specs::prelude::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let dest_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[dest_idx] != TileType::Wall {
            pos.x = (pos.x + delta_x).clamp(0, WINDOW_WIDTH - 1);
            pos.y = (pos.y + delta_y).clamp(0, WINDOW_HEIGHT - 1);
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) {
    use VirtualKeyCode::*;
    if let Some(key) = ctx.key {
        match key {
            Left | Numpad4 | H => try_move_player(-1, 0, &mut gs.ecs),
            Right | Numpad6 | L => try_move_player(1, 0, &mut gs.ecs),
            Up | Numpad8 | K => try_move_player(0, -1, &mut gs.ecs),
            Down | Numpad2 | J => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        };
    }
}
