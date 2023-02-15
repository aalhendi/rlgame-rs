use rltk::{Rltk, RGB};
use crate::SPAWN_X;
use crate::SPAWN_Y;
use crate::WINDOW_HEIGHT;
use crate::WINDOW_WIDTH;

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub fn xy_idx(x: i32, y: i32) -> usize {
    (y * WINDOW_WIDTH + x) as usize
}

pub fn new_map() -> Vec<TileType> {
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
    let mut rng = rltk::RandomNumberGenerator::new();
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
