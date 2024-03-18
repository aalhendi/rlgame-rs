use rltk::RandomNumberGenerator;

use super::{BuilderMap, InitialMapBuilder, MetaMapBuilder};
use crate::{Position, TileType};
use std::collections::HashSet;
pub mod prefab_levels;
pub mod prefab_rooms;
pub mod prefab_sections;

pub enum PrefabMode {
    RexLevel {
        template: &'static str,
    },
    Constant {
        level: prefab_levels::PrefabLevel,
    },
    Sectional {
        section: prefab_sections::PrefabSection,
    },
    RoomVaults,
}

pub struct PrefabBuilder {
    mode: PrefabMode,
}

impl MetaMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl InitialMapBuilder for PrefabBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl PrefabBuilder {
    #[allow(dead_code)]
    pub fn rex_level(template: &'static str) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::RexLevel { template },
        })
    }

    pub fn constant(level: prefab_levels::PrefabLevel) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::Constant { level },
        })
    }

    pub fn sectional(section: prefab_sections::PrefabSection) -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::Sectional { section },
        })
    }

    pub fn vaults() -> Box<PrefabBuilder> {
        Box::new(PrefabBuilder {
            mode: PrefabMode::RoomVaults,
        })
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(template, build_data),
            PrefabMode::Constant { level } => self.load_ascii_map(&level, build_data),
            PrefabMode::Sectional { section } => self.apply_sectional(&section, rng, build_data),
            PrefabMode::RoomVaults => self.apply_room_vaults(rng, build_data),
        }
        build_data.take_snapshot();
    }

    // MIT LICENSE: @Smokku - Copyright (c) 2020 Tomasz Sterna
    fn read_ascii_to_vec(template: &str, width: usize, height: usize) -> Vec<char> {
        let vec: Vec<char> = template
            .lines()
            .map(|line| format!("{line: <width$}"))
            .collect::<Vec<_>>()
            .concat()
            .chars()
            .collect();
        if vec.len() != width * height {
            panic!("Loaded template did not yield the expected number of characters. Got {cur_chars}, expected {total_chars}.\n{template:?}", cur_chars = vec.len(), total_chars = width * height);
        }

        vec
    }

    fn load_rex_map(&mut self, path: &str, build_data: &mut BuilderMap) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < build_data.map.width as usize && y < build_data.map.height as usize {
                        let idx = build_data.map.xy_idx(x as i32, y as i32);
                        // NOTE: nasty casting.
                        self.set_tile_from_char(cell.ch as u8 as char, idx, build_data);
                    }
                }
            }
        }
    }

    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel, build_data: &mut BuilderMap) {
        // Start by converting to a vector, with newlines removed
        let string_vec =
            PrefabBuilder::read_ascii_to_vec(level.template, level.width, level.height);

        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < build_data.map.width as usize && ty < build_data.map.height as usize {
                    let idx = build_data.map.xy_idx(tx as i32, ty as i32);
                    self.set_tile_from_char(string_vec[i], idx, build_data);
                }
                i += 1;
            }
        }
    }

    // TODO(aalhendi): Refactor
    fn set_tile_from_char(&mut self, ch: char, idx: usize, build_data: &mut BuilderMap) {
        let tile = match ch {
            ' ' | '@' | 'g' | '^' | '!' | '%' | 'O' | '☼' => Some(TileType::Floor),
            '#' => Some(TileType::Wall),
            '>' => Some(TileType::DownStairs),
            _ => None,
        };
        if let Some(tile) = tile {
            build_data.map.tiles[idx] = tile;
        }

        let spawn = match ch {
            'g' => Some("Goblin".to_string()),
            //'o' => Some(Monster::Orc), // TODO: Use enum for spawns. to_string() allocs to heap
            'o' => Some("Orc".to_string()),
            'O' => Some("Orc Leader".to_string()),
            '^' => Some("Bear Trap".to_string()),
            '%' => Some("Rations".to_string()),
            '!' => Some("Health Potion".to_string()),
            '☼' => Some("Watch Fire".to_string()),
            _ => None,
        };
        if let Some(name) = spawn {
            build_data.spawn_list.push((idx, name));
        }

        if ch == '@' {
            // Player obviously stands on a floor tile
            let (x, y) = build_data.map.idx_xy(idx);
            build_data.starting_position = Some(Position { x, y });
        }
    }

    fn apply_previous_iteration<F>(
        &mut self,
        mut filter: F,
        _rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) where
        F: FnMut(i32, i32) -> bool,
    {
        build_data.spawn_list.retain(|(idx, _name)| {
            let (x, y) = build_data.map.idx_xy(*idx);
            filter(x, y)
        });
        build_data.take_snapshot();
    }

    fn apply_sectional(
        &mut self,
        section: &prefab_sections::PrefabSection,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        use prefab_sections::*;

        let string_vec =
            PrefabBuilder::read_ascii_to_vec(section.template, section.width, section.height);

        // Place the new section
        let chunk_x = match section.placement.0 {
            HorizontalPlacement::Left => 0,
            HorizontalPlacement::Center => (build_data.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => (build_data.map.width - 1) - section.width as i32,
        };

        let chunk_y = match section.placement.1 {
            VerticalPlacement::Top => 0,
            VerticalPlacement::Center => (build_data.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => (build_data.map.height - 1) - section.height as i32,
        };

        // Build the map
        self.apply_previous_iteration(
            |x, y| {
                x < chunk_x
                    || x > (chunk_x + section.width as i32)
                    || y < chunk_y
                    || y > (chunk_y + section.height as i32)
            },
            rng,
            build_data,
        );

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx > 0
                    && tx < build_data.map.width as usize - 1
                    && ty < build_data.map.height as usize - 1
                    && ty > 0
                {
                    let idx = build_data
                        .map
                        .xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.set_tile_from_char(string_vec[i], idx, build_data);
                }
                i += 1;
            }
        }
        build_data.take_snapshot();
    }

    fn apply_room_vaults(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        use prefab_rooms::*;

        // Apply the previous builder, and keep all entities it spawns (for now)
        self.apply_previous_iteration(|_x, _y| true, rng, build_data);

        // Vault spawns have 3/6 chance
        let vault_roll = rng.roll_dice(1, 6) + build_data.map.depth;
        if vault_roll < 4 {
            return;
        };

        // NOTE: place-holder, will be moved out of this function
        let master_vault_list = [TOTALLY_NOT_A_TRAP, SILLY_SMILE, CHECKERBOARD];

        // Filter vault list by current depth
        let mut possible_vaults: Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| {
                build_data.map.depth >= v.first_depth && build_data.map.depth <= v.last_depth
            })
            .collect();

        // No vaults, return early
        if possible_vaults.is_empty() {
            return;
        }

        let n_vaults = i32::min(rng.roll_dice(1, 3), possible_vaults.len() as i32);
        let mut used_tiles = HashSet::new();

        for _ in 0..n_vaults {
            let vault_index = if possible_vaults.len() == 1 {
                0
            } else {
                (rng.roll_dice(1, possible_vaults.len() as i32) - 1) as usize
            };
            let vault = possible_vaults[vault_index];

            // List of positions vault can fit it
            let mut vault_positions: Vec<Position> = Vec::new();

            for idx in 0..build_data.map.tiles.len() {
                let (x, y) = build_data.map.idx_xy(idx);

                // Check map bounds
                if x > 1
                    && (x + vault.width as i32) < build_data.map.width - 2
                    && y > 1
                    && (y + vault.height as i32) < build_data.map.height - 2
                {
                    // TODO: REFACTOR
                    let mut possible = true;
                    'map: for ty in 0..vault.height as i32 {
                        for tx in 0..vault.width as i32 {
                            let t_idx = build_data.map.xy_idx(tx + x, ty + y);
                            if build_data.map.tiles[t_idx] != TileType::Floor
                                || used_tiles.contains(&idx)
                            {
                                possible = false;
                                break 'map;
                            }
                        }
                    }

                    if possible {
                        vault_positions.push(Position { x, y });
                        break;
                    }
                }
            }

            if vault_positions.is_empty() {
                continue;
            }

            let pos_idx = if vault_positions.len() == 1 {
                0
            } else {
                (rng.roll_dice(1, vault_positions.len() as i32) - 1) as usize
            };
            let pos = &vault_positions[pos_idx];

            // Don't spawn things in the vault tiles.
            build_data.spawn_list.retain(|e| {
                let idx = e.0 as i32;
                let (x, y) = build_data.map.idx_xy(idx as usize);
                x < pos.x
                    || x > pos.x + vault.width as i32
                    || y < pos.y
                    || y > pos.y + vault.height as i32
            });

            let string_vec =
                PrefabBuilder::read_ascii_to_vec(vault.template, vault.width, vault.height);
            let mut i = 0;
            for ty in 0..vault.height {
                for tx in 0..vault.width {
                    let idx = build_data.map.xy_idx(tx as i32 + pos.x, ty as i32 + pos.y);
                    self.set_tile_from_char(string_vec[i], idx, build_data);
                    used_tiles.insert(idx);
                    i += 1;
                }
            }
            build_data.take_snapshot();
            possible_vaults.remove(vault_index);
        }
    }
}
