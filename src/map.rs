use super::Rect;
use rltk::{Point, RandomNumberGenerator, Rltk, RGB};
use specs::{Entity, World};
use std::cmp::{max, min};
use std::collections::HashSet;

pub const MAPWIDTH: usize = 80;
pub const MAPHEIGHT: usize = 43;
pub const MAPCOUNT: usize = MAPWIDTH * MAPHEIGHT;

#[derive(PartialEq, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
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

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        // Diagonal directions
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push((idx - w - 1, 1.45))
        };
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push((idx - w + 1, 1.45))
        };
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push((idx + w - 1, 1.45))
        };
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push((idx + w + 1, 1.45))
        };

        exits
    }
}

impl Map {
    /// Returns index in 1D array via row-major indexing
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    // Clears the contents of tile_content field
    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    /// Returns if a tile can be entered and is within bounds
    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        // Check boundaries & out of bounds
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    /// Sets tile as blocked if Wall tile.
    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    /// Returns a map with solid boundaries and 400 randomly placed wall tiles
    pub fn new_map_test(&self) -> Vec<TileType> {
        let mut map = vec![TileType::Floor; MAPCOUNT];

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

    /// Makes a new map using the algorithm from <http://rogueliketutorials.com/tutorials/tcod/part-3/>
    /// Returns map with random rooms and corridors to join them.
    pub fn new_map_rooms_and_corridors(new_depth: i32) -> Map {
        let mut map = Map {
            tiles: vec![TileType::Wall; MAPCOUNT],
            rooms: Vec::new(),
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            tile_content: vec![Vec::new(); MAPCOUNT],
            depth: new_depth,
            bloodstains: HashSet::new(),
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

        // Insert down stairs in center of last room
        let stairs_pos = map.rooms[map.rooms.len() - 1].center();
        let stairs_idx = map.xy_idx(stairs_pos.x, stairs_pos.y);
        map.tiles[stairs_idx] = TileType::DownStairs;

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

    fn is_revealed_and_wall(&self, x: i32, y: i32) -> bool {
        let idx = self.xy_idx(x, y);
        self.tiles[idx] == TileType::Wall && self.revealed_tiles[idx]
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
            let mut bg = RGB::from_f32(0., 0., 0.);
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.5, 0.5);
                }
                TileType::Wall => {
                    glyph = get_wall_glyph(&*map, x, y);
                    fg = RGB::from_f32(0., 1.0, 0.);
                }
                TileType::DownStairs => {
                    glyph = rltk::to_cp437('>');
                    fg = RGB::from_f32(0.0, 1.0, 1.0);
                }
            }
            if map.bloodstains.contains(&idx) {
                bg = RGB::from_f32(0.75, 0.0, 0.0)
            }
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale()
            }
            ctx.set(x, y, fg, bg, glyph);
        }

        // iter coordinates as well
        x += 1;
        if x > map.width - 1 {
            x = 0;
            y += 1;
        }
    }
}

fn get_wall_glyph(map: &Map, x: i32, y: i32) -> rltk::FontCharType {
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2 as i32 {
        return 35;
    }
    let mut mask: u8 = 0;

    if map.is_revealed_and_wall(x, y - 1) {
        mask += 1;
    }
    if map.is_revealed_and_wall(x, y + 1) {
        mask += 2;
    }
    if map.is_revealed_and_wall(x - 1, y) {
        mask += 4;
    }
    if map.is_revealed_and_wall(x + 1, y) {
        mask += 8;
    }

    // Uses <http://dwarffortresswiki.org/index.php/Character_table>
    match mask {
        0 => 9,    // Pillar because we can't see neighbors
        1 => 186,  // Wall only to the north
        2 => 186,  // Wall only to the south
        3 => 186,  // Wall to the north and south
        4 => 205,  // Wall only to the west
        5 => 188,  // Wall to the north and west
        6 => 187,  // Wall to the south and west
        7 => 185,  // Wall to the north, south and west
        8 => 205,  // Wall only to the east
        9 => 200,  // Wall to the north and east
        10 => 201, // Wall to the south and east
        11 => 204, // Wall to the north, south and east
        12 => 205, // Wall to the east and west
        13 => 202, // Wall to the east, west, and south
        14 => 203, // Wall to the east, west, and north
        15 => 206, // ╬ Wall on all sides
        _ => 35,   // Fallthrough...
    }
}
