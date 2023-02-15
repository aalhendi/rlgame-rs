use super::Rect;
use crate::SPAWN_X;
use crate::SPAWN_Y;
use crate::WINDOW_HEIGHT;
use crate::WINDOW_WIDTH;
use rltk::{RandomNumberGenerator, Rltk, RGB};
use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y * WINDOW_WIDTH + x) as usize
}
/// Returns a map with solid boundaries and 400 randomly placed wall tiles
pub fn new_map_test() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize];

    // Setting window boundaries as walls
    for x in 0..WINDOW_WIDTH {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, WINDOW_HEIGHT - 1)] = TileType::Wall;
    }
    for y in 0..WINDOW_HEIGHT {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(WINDOW_WIDTH - 1, y)] = TileType::Wall;
    }

    // Random Walls on ~10% of tiles via thread-local rng
    let mut rng = RandomNumberGenerator::new();
    let spawn_idx = xy_idx(SPAWN_X, SPAWN_Y);
    for _ in 0..400 {
        let x = rng.roll_dice(1, WINDOW_WIDTH - 1);
        let y = rng.roll_dice(1, WINDOW_HEIGHT - 1);
        let idx = xy_idx(x, y);
        if idx != spawn_idx {
            map[idx] = TileType::Wall;
        }
    }

    map
}

pub fn new_map_rooms_and_corridors() -> Vec<TileType> {
    let mut map = vec![TileType::Wall; (WINDOW_HEIGHT * WINDOW_WIDTH) as usize];

    let room1 = Rect::new(20, 15, 10, 15);
    let room2 = Rect::new(35, 15, 10, 15);

    apply_room_to_map(&room1, &mut map);
    apply_room_to_map(&room2, &mut map);

    apply_horizontal_tunnel(&mut map, 25, 40, 23);

    map
}
pub fn apply_room_to_map(room: &Rect, map: &mut [TileType]) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            map[xy_idx(x, y)] = TileType::Floor;
        }
    }
}

pub fn apply_horizontal_tunnel(map: &mut [TileType], x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < (WINDOW_WIDTH * WINDOW_HEIGHT) as usize {
            map[idx] = TileType::Floor;
        }
    }
}
pub fn apply_vertical_tunnel(map: &mut [TileType], y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = xy_idx(x, y);
        if idx > 0 && idx < (WINDOW_WIDTH * WINDOW_HEIGHT) as usize {
            map[idx] = TileType::Floor;
        }
    }
}

pub fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        match tile {
            TileType::Floor => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::named(rltk::BLACK),
                    rltk::to_cp437('.'),
                );
            }
            TileType::Wall => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::named(rltk::BLACK),
                    rltk::to_cp437('#'),
                );
            }
        }

        // iter coordinates as well
        x += 1;
        if x > WINDOW_WIDTH - 1 {
            x = 0;
            y += 1;
        }
    }
}
