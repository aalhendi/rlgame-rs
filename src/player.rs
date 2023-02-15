use specs::prelude::*;
use rltk::{VirtualKeyCode, Rltk};
use super::{Position, Player, TileType, xy_idx, State};
use crate::WINDOW_HEIGHT;
use crate::WINDOW_WIDTH;

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
    if let Some(key) = ctx.key {
        match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        };
    }
}
