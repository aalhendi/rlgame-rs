use rltk::{to_cp437, FontCharType, RGB};

use crate::{camera::PANE_WIDTH, Map, TileType};

pub fn tile_glyph(idx: usize, map: &Map) -> (FontCharType, RGB, RGB) {
    let (glyph, mut fg, mut bg) = match map.depth {
        8 | 9 => get_mushroom_glyph(idx, map),
        7 => {
            let (x, _y) = map.idx_xy(idx);
            if x > map.width - (PANE_WIDTH as f32 / 2.75) as i32 {
                get_tile_glyph_default(idx, map)
            } else {
                get_mushroom_glyph(idx, map)
            }
        }
        5 => {
            let (x, _y) = map.idx_xy(idx);
            if x < map.width / 2 {
                get_limestone_cavern_glyph(idx, map)
            } else {
                get_tile_glyph_default(idx, map)
            }
        }
        4 => get_limestone_cavern_glyph(idx, map),
        3 => get_limestone_cavern_glyph(idx, map),
        2 => get_forest_glyph(idx, map),
        _ => get_tile_glyph_default(idx, map),
    };

    if map.bloodstains.contains(&idx) {
        bg = RGB::from_f32(0.75, 0., 0.);
    }

    // If can't tile, use greyscales
    if !map.visible_tiles[idx] {
        fg = fg.to_greyscale();
        // Don't show stains out of visual range
        bg = RGB::from_f32(0., 0., 0.);
    // If can see tile && outdoors is false - multiply colors light intensity
    } else if !map.outdoors {
        fg = fg * map.light_level_tiles[idx];
        bg = bg * map.light_level_tiles[idx];
    }

    (glyph, fg, bg)
}

// TODO(aalhendi): Should this be part of impl Map?
fn get_tile_glyph_default(idx: usize, map: &Map) -> (FontCharType, RGB, RGB) {
    let glyph;
    let fg;
    let bg = RGB::from_f32(0., 0., 0.);

    match map.tiles[idx] {
        TileType::Floor => {
            glyph = to_cp437('.');
            fg = RGB::from_f32(0.0, 0.5, 0.5);
        }
        TileType::WoodFloor => {
            glyph = to_cp437('░');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Wall => {
            let (x, y) = map.idx_xy(idx);
            glyph = get_wall_glyph(map, x, y);
            fg = RGB::named(rltk::GREEN);
        }
        TileType::DownStairs => {
            glyph = to_cp437('>');
            fg = RGB::named(rltk::CYAN);
        }
        TileType::Bridge => {
            glyph = to_cp437('.');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Road => {
            glyph = to_cp437('≡');
            fg = RGB::named(rltk::GRAY);
        }
        TileType::Grass => {
            glyph = to_cp437('"');
            fg = RGB::named(rltk::GREEN);
        }
        TileType::ShallowWater => {
            glyph = to_cp437('~');
            fg = RGB::named(rltk::CYAN); // BLUE
        }
        TileType::DeepWater => {
            glyph = to_cp437('≈');
            fg = RGB::named(rltk::NAVYBLUE);
        }
        TileType::Gravel => {
            glyph = to_cp437(';');
            fg = RGB::named(rltk::WEBGRAY);
        }
        TileType::UpStairs => {
            glyph = to_cp437('<');
            fg = RGB::named(rltk::CYAN);
        }
        TileType::Stalactite => {
            glyph = rltk::to_cp437('╨');
            fg = RGB::named(rltk::WEBGRAY);
        }
        TileType::Stalagmite => {
            glyph = rltk::to_cp437('╥');
            fg = RGB::named(rltk::WEBGRAY);
        }
    }

    (glyph, fg, bg)
}

// TODO(aalhendi): Should this be part of impl Map?
fn get_forest_glyph(idx: usize, map: &Map) -> (FontCharType, RGB, RGB) {
    let glyph;
    let fg;
    let bg = RGB::named(rltk::BLACK);

    match map.tiles[idx] {
        TileType::Wall => {
            glyph = to_cp437('♣');
            fg = RGB::from_f32(0.0, 0.6, 0.0);
        }
        TileType::Bridge => {
            glyph = to_cp437('.');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Road => {
            glyph = to_cp437('≡');
            fg = RGB::named(rltk::YELLOW);
        }
        TileType::Grass => {
            glyph = to_cp437('"');
            fg = RGB::named(rltk::GREEN);
        }
        TileType::ShallowWater => {
            glyph = to_cp437('~');
            fg = RGB::named(rltk::CYAN);
        }
        TileType::DeepWater => {
            glyph = to_cp437('≈');
            fg = RGB::named(rltk::NAVYBLUE);
        }
        TileType::Gravel => {
            glyph = to_cp437(';');
            fg = RGB::from_f32(0.5, 0.5, 0.5);
        }
        TileType::DownStairs => {
            glyph = to_cp437('>');
            fg = RGB::named(rltk::CYAN);
        }
        TileType::UpStairs => {
            glyph = to_cp437('<');
            fg = RGB::named(rltk::CYAN);
        }
        _ => {
            glyph = to_cp437('"');
            fg = RGB::from_f32(0.0, 0.6, 0.0);
        }
    }

    (glyph, fg, bg)
}

fn get_limestone_cavern_glyph(idx: usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
    let glyph;
    let fg;
    let bg = RGB::from_f32(0., 0., 0.);

    match map.tiles[idx] {
        TileType::Wall => {
            glyph = to_cp437('▒');
            fg = RGB::from_f32(0.7, 0.7, 0.7);
        }
        TileType::Bridge => {
            glyph = to_cp437('.');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Road => {
            glyph = to_cp437('≡');
            fg = RGB::named(rltk::YELLOW);
        }
        TileType::Grass => {
            glyph = to_cp437('"');
            fg = RGB::named(rltk::GREEN);
        }
        TileType::ShallowWater => {
            glyph = to_cp437('░');
            fg = RGB::named(rltk::CYAN);
        }
        TileType::DeepWater => {
            glyph = to_cp437('▓');
            // fg = RGB::named(rltk::BLUE);
            fg = RGB::from_f32(0.2, 0.2, 1.0) // hint of green to see it in greyscale
        }
        TileType::Gravel => {
            glyph = to_cp437(';');
            fg = RGB::from_f32(0.5, 0.5, 0.5);
        }
        TileType::DownStairs => {
            glyph = to_cp437('>');
            fg = RGB::named(rltk::CYAN);
        }
        TileType::UpStairs => {
            glyph = to_cp437('<');
            fg = RGB::named(rltk::CYAN);
        }
        TileType::Stalactite => {
            glyph = rltk::to_cp437('╨');
            fg = RGB::named(rltk::WEBGRAY);
        }
        TileType::Stalagmite => {
            glyph = rltk::to_cp437('╥');
            fg = RGB::named(rltk::WEBGRAY);
        }
        _ => {
            glyph = to_cp437('░');
            fg = RGB::from_f32(0.4, 0.4, 0.4);
        }
    }

    (glyph, fg, bg)
}

fn get_mushroom_glyph(idx: usize, map: &Map) -> (rltk::FontCharType, RGB, RGB) {
    let glyph;
    let fg;
    let bg = RGB::from_f32(0., 0., 0.);

    match map.tiles[idx] {
        TileType::Wall => {
            glyph = rltk::to_cp437('♠');
            fg = RGB::from_f32(1.0, 0.0, 1.0);
        }
        TileType::Bridge => {
            glyph = rltk::to_cp437('.');
            fg = RGB::named(rltk::GREEN);
        }
        TileType::Road => {
            glyph = rltk::to_cp437('≡');
            fg = RGB::named(rltk::CHOCOLATE);
        }
        TileType::Grass => {
            glyph = rltk::to_cp437('"');
            fg = RGB::named(rltk::GREEN);
        }
        TileType::ShallowWater => {
            glyph = rltk::to_cp437('~');
            fg = RGB::named(rltk::CYAN);
        }
        TileType::DeepWater => {
            glyph = rltk::to_cp437('≈');
            fg = RGB::named(rltk::BLUE);
        }
        TileType::Gravel => {
            glyph = rltk::to_cp437(';');
            fg = RGB::from_f32(0.5, 0.5, 0.5);
        }
        TileType::DownStairs => {
            glyph = rltk::to_cp437('>');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        TileType::UpStairs => {
            glyph = rltk::to_cp437('<');
            fg = RGB::from_f32(0., 1.0, 1.0);
        }
        _ => {
            glyph = rltk::to_cp437('"');
            fg = RGB::from_f32(0.0, 0.6, 0.0);
        }
    }

    (glyph, fg, bg)
}

// TODO(aalhendi): Should this be part of impl Map?
pub fn get_wall_glyph(map: &Map, x: i32, y: i32) -> FontCharType {
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
        _ => 35,   // Fallthrough
    }
}
