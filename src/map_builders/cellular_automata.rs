use crate::map::TileType;

use super::{BuilderMap, InitialMapBuilder};

use rltk::RandomNumberGenerator;

/// Inspired by: <http://www.roguebasin.com/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels>
pub struct CellularAutomataBuilder {}

impl InitialMapBuilder for CellularAutomataBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl CellularAutomataBuilder {
    pub fn new() -> Box<CellularAutomataBuilder> {
        Box::new(CellularAutomataBuilder {})
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        // Create random map, 55% floor. Cellular automata are designed to make a level out of noise
        for y in 1..build_data.map.height - 1 {
            for x in 1..build_data.map.width - 1 {
                let roll = rng.roll_dice(1, 100);
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = if roll > 55 {
                    TileType::Floor
                } else {
                    TileType::Wall
                }
            }
            build_data.take_snapshot();
        }

        // Now we iteratively apply cellular automata rules:
        // iterating each cell,
        // counting the number of neighbors,
        // and turning walls into floors or walls based on density.
        let map_width = build_data.map.width as usize;
        for _ in 0..15 {
            // Used to not write on the tiles we are counting, which gives a very odd map...
            let mut newtiles = build_data.map.tiles.clone();

            for y in 1..build_data.map.height - 1 {
                for x in 1..build_data.map.width - 1 {
                    let idx = build_data.map.xy_idx(x, y);
                    // TODO: Refactor
                    let mut neighbors = 0;
                    if build_data.map.tiles[idx - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx - map_width] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx + map_width] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx - map_width - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx - map_width + 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx + map_width - 1] == TileType::Wall {
                        neighbors += 1;
                    }
                    if build_data.map.tiles[idx + map_width + 1] == TileType::Wall {
                        neighbors += 1;
                    }

                    if neighbors > 4 || neighbors == 0 {
                        newtiles[idx] = TileType::Wall;
                    } else {
                        newtiles[idx] = TileType::Floor;
                    }
                }
            }

            build_data.map.tiles = newtiles.clone();
            build_data.take_snapshot();
        }
    }
}
