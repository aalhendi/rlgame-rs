use super::Rect;
use crate::WINDOW_HEIGHT;
use crate::WINDOW_WIDTH;
use rltk::{RandomNumberGenerator, Rltk, RGB};
use specs::World;
use std::cmp::{max, min};

#[derive(PartialEq, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
}

pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
}

impl rltk::Algorithm2D for Map {
    fn dimensions(&self) -> rltk::Point {
        rltk::Point::new(self.width, self.height)
    }
}

impl rltk::BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    /// Returns a map with solid boundaries and 400 randomly placed wall tiles
    pub fn new_map_test(&self) -> Vec<TileType> {
        let mut map = vec![TileType::Floor; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize];

        // Setting window boundaries as walls
        for x in 0..self.width {
            map[self.xy_idx(x, 0)] = TileType::Wall;
            map[self.xy_idx(x, self.height - 1)] = TileType::Wall;
        }
        for y in 0..self.height {
            map[self.xy_idx(0, y)] = TileType::Wall;
            map[self.xy_idx(self.width - 1, y)] = TileType::Wall;
        }

        // Random Walls on ~10% of tiles via thread-local rng
        let mut rng = RandomNumberGenerator::new();
        let spawn_idx = self.xy_idx(40, 25);
        for _ in 0..400 {
            let x = rng.roll_dice(1, self.width - 1);
            let y = rng.roll_dice(1, self.height - 1);
            let idx = self.xy_idx(x, y);
            if idx != spawn_idx {
                map[idx] = TileType::Wall;
            }
        }

        map
    }

    /// Makes a new map using the algorithm from http://rogueliketutorials.com/tutorials/tcod/part-3/
    /// Returns map with random rooms and corridors to join them.
    pub fn new_map_rooms_and_corridors() -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; (WINDOW_HEIGHT * WINDOW_WIDTH) as usize],
            rooms: Vec::new(),
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            revealed_tiles: vec![false; (WINDOW_HEIGHT * WINDOW_WIDTH) as usize],
            visible_tiles: vec![false; (WINDOW_HEIGHT * WINDOW_WIDTH) as usize],
        };

        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;

        let mut rng = RandomNumberGenerator::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - 1 - w) - 1;
            let y = rng.roll_dice(1, map.height - 1 - h) - 1;

            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersects(other_room) {
                    ok = false;
                }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let new_center = new_room.center();
                    let old_center = map.rooms[map.rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        map.apply_horizontal_tunnel(old_center.x, new_center.x, old_center.y);
                        map.apply_vertical_tunnel(old_center.y, new_center.y, new_center.x);
                    } else {
                        map.apply_vertical_tunnel(old_center.y, new_center.y, new_center.x);
                        map.apply_horizontal_tunnel(old_center.x, new_center.x, old_center.y);
                    }
                }

                map.rooms.push(new_room)
            }
        }

        map
    }
    pub fn apply_room_to_map(&mut self, room: &Rect) {
        for y in room.y1 + 1..=room.y2 {
            for x in room.x1 + 1..=room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    pub fn apply_horizontal_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < (self.width * self.height) as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }
    pub fn apply_vertical_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < (self.width * self.height) as usize {
                self.tiles[idx] = TileType::Floor;
            }
        }
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::Wall => {
                    glyph = rltk::to_cp437('#');
                    fg = RGB::from_f32(0.0, 1.0, 0.0);
                }
            }
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale()
            }
            ctx.set(x, y, fg, RGB::named(rltk::BLACK), glyph);
        }

        // iter coordinates as well
        x += 1;
        if x > map.width - 1 {
            x = 0;
            y += 1;
        }
    }
}
