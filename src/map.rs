use super::Rect;
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
    let spawn_idx = xy_idx(40, 25);
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

/// Makes a new map using the algorithm from http://rogueliketutorials.com/tutorials/tcod/part-3/
/// Returns map with random rooms and corridors to join them.
pub fn new_map_rooms_and_corridors() -> (Vec<Rect>, Vec<TileType>) {
    let mut map = vec![TileType::Wall; (WINDOW_HEIGHT * WINDOW_WIDTH) as usize];

    let mut rooms: Vec<Rect> = Vec::new();
    const MAX_ROOMS: i32 = 30;
    const MIN_SIZE: i32 = 6;
    const MAX_SIZE: i32 = 10;

    let mut rng = RandomNumberGenerator::new();

    for _ in 0..MAX_ROOMS {
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, WINDOW_WIDTH - 1 - w) - 1;
        let y = rng.roll_dice(1, WINDOW_HEIGHT - 1 - h) - 1;

        let new_room = Rect::new(x, y, w, h);
        let mut ok = true;
        for other_room in rooms.iter() {
            if new_room.intersects(other_room) {
                ok = false;
            }
        }
        if ok {
            apply_room_to_map(&new_room, &mut map);

            if !rooms.is_empty() {
                let new_center = new_room.center();
                let old_center = rooms[rooms.len() - 1].center();
                if rng.range(0, 2) == 1 {
                    apply_horizontal_tunnel(&mut map, old_center.x, new_center.x, old_center.y);
                    apply_vertical_tunnel(&mut map, old_center.y, new_center.y, old_center.x);
                } else {
                    apply_horizontal_tunnel(&mut map, old_center.x, new_center.x, new_center.y);
                    apply_vertical_tunnel(&mut map, old_center.y, new_center.y, old_center.x);
                }
            }

            rooms.push(new_room)
        }
    }

    (rooms, map)
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
