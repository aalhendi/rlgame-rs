use rltk::{Point, RandomNumberGenerator};
use specs::Entity;
use std::collections::HashSet;

#[derive(PartialEq, Eq, Hash, Copy, Clone, serde::Serialize, serde::Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Default)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: i32,
    pub bloodstains: HashSet<usize>,
    pub view_blocked: HashSet<usize>,

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
        self.tiles[idx] == TileType::Wall || self.view_blocked.contains(&idx)
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let (x, y) = self.idx_xy(idx);
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
    /// Generates an empty map, consisting entirely of solid walls
    pub fn new(new_depth: i32, width: i32, height: i32) -> Map {
        let map_tile_count = (width * height) as usize;
        Map {
            tiles: vec![TileType::Wall; map_tile_count],
            width,
            height,
            revealed_tiles: vec![false; map_tile_count],
            visible_tiles: vec![false; map_tile_count],
            blocked: vec![false; map_tile_count],
            tile_content: vec![Vec::new(); map_tile_count],
            depth: new_depth,
            bloodstains: HashSet::new(),
            view_blocked: HashSet::new(),
        }
    }

    /// Returns index in 1D array via row-major indexing
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y * self.width + x) as usize
    }

    /// Returns x, y coordinates given an array index
    pub fn idx_xy(&self, idx: usize) -> (i32, i32) {
        (idx as i32 % self.width, idx as i32 / self.width)
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
        let map_tile_count = (self.width * self.height) as usize;
        let mut map = vec![TileType::Floor; map_tile_count];

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

    fn is_revealed_and_wall(&self, x: i32, y: i32) -> bool {
        let idx = self.xy_idx(x, y);
        self.tiles[idx] == TileType::Wall && self.revealed_tiles[idx]
    }
}

pub fn get_wall_glyph(map: &Map, x: i32, y: i32) -> rltk::FontCharType {
    if x < 1 || x > map.width - 2 || y < 1 || y > map.height - 2 {
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
