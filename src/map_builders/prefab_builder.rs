use super::common::remove_unreachable_areas_get_most_distant;
use super::{Map, MapBuilder};
use crate::{spawner, Position, TileType, SHOW_MAPGEN_VISUALIZER};
use specs::World;

#[derive(PartialEq, Eq)]
pub enum PrefabMode {
    RexLevel { template: &'static str },
}

pub struct PrefabBuilder {
    map: Map,
    starting_position: Position,
    _depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    spawns: Vec<(usize, String)>,
}

impl MapBuilder for PrefabBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn get_snapshot_history(&self) -> Vec<Map> {
        self.history.clone()
    }

    fn build_map(&mut self) {
        self.build();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        for (idx, name) in self.spawns.iter() {
            spawner::spawn_entity(ecs, &(idx, name));
        }
    }

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }
}

impl PrefabBuilder {
    pub fn new(new_depth: i32) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            _depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::RexLevel {
                template: "../../resources/wfc-populated.xp",
            },
            spawns: Vec::new(),
        }
    }

    fn build(&mut self) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(template),
        }
        self.take_snapshot();

        if self.starting_position.x == 0 {
            self.starting_position = Position {
                x: self.map.width / 2,
                y: self.map.height / 2,
            };
            let mut start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y);
            while self.map.tiles[start_idx] != TileType::Floor {
                self.starting_position.x -= 1;
                start_idx = self
                    .map
                    .xy_idx(self.starting_position.x, self.starting_position.y);
            }
            self.take_snapshot();

            let exit_tile = remove_unreachable_areas_get_most_distant(&mut self.map, start_idx);
            self.take_snapshot();

            // Place the stairs
            self.map.tiles[exit_tile] = TileType::DownStairs;
            self.take_snapshot();
        }
    }

    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = self.map.xy_idx(x as i32, y as i32);
                        // nasty casting to match chars instead of CP-437 numbers
                        match (cell.ch as u8) as char {
                            ' ' => self.map.tiles[idx] = TileType::Floor,
                            '#' => self.map.tiles[idx] = TileType::Wall,
                            '@' => {
                                // Player obviously stands on a floor tile
                                self.map.tiles[idx] = TileType::Floor;
                                self.starting_position = Position {
                                    x: x as i32,
                                    y: y as i32,
                                };
                            }
                            '>' => self.map.tiles[idx] = TileType::DownStairs,
                            'g' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Goblin".to_string()));
                            }
                            'o' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Orc".to_string()));
                            }
                            '^' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Bear Trap".to_string()));
                            }
                            '%' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Rations".to_string()));
                            }
                            '!' => {
                                self.map.tiles[idx] = TileType::Floor;
                                self.spawns.push((idx, "Health Potion".to_string()));
                            }
                            _ => {
                                rltk::console::log(format!(
                                    "Unknown glyph loading map: {}",
                                    (cell.ch as u8) as char
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}
