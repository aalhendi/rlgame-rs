use super::common::remove_unreachable_areas_get_most_distant;
use super::{Map, MapBuilder};
use crate::{Position, TileType, SHOW_MAPGEN_VISUALIZER};
use std::collections::HashSet;
pub mod prefab_levels;
pub mod prefab_rooms;
pub mod prefab_sections;

#[allow(dead_code)]
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
    map: Map,
    starting_position: Position,
    depth: i32,
    history: Vec<Map>,
    mode: PrefabMode,
    previous_builder: Option<Box<dyn MapBuilder>>,
    spawn_list: Vec<(usize, String)>,
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

    fn take_snapshot(&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            let mut snapshot = self.map.clone();
            for v in snapshot.revealed_tiles.iter_mut() {
                *v = true;
            }
            self.history.push(snapshot);
        }
    }

    fn get_spawn_list(&self) -> &Vec<(usize, String)> {
        &self.spawn_list
    }
}

impl PrefabBuilder {
    pub fn new(new_depth: i32, previous_builder: Option<Box<dyn MapBuilder>>) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::RoomVaults,
            previous_builder,
            spawn_list: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn rex_level(new_depth: i32, template: &'static str) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::RexLevel { template },
            previous_builder: None,
            spawn_list: Vec::new(),
        }
    }

    pub fn constant(new_depth: i32, level: prefab_levels::PrefabLevel) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::Constant { level },
            previous_builder: None,
            spawn_list: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn sectional(
        new_depth: i32,
        section: prefab_sections::PrefabSection,
        previous_builder: Box<dyn MapBuilder>,
    ) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::Sectional { section },
            previous_builder: Some(previous_builder),
            spawn_list: Vec::new(),
        }
    }

    pub fn vaults(new_depth: i32, previous_builder: Box<dyn MapBuilder>) -> PrefabBuilder {
        PrefabBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            history: Vec::new(),
            mode: PrefabMode::RoomVaults,
            previous_builder: Some(previous_builder),
            spawn_list: Vec::new(),
        }
    }

    fn build(&mut self) {
        match self.mode {
            PrefabMode::RexLevel { template } => self.load_rex_map(template),
            PrefabMode::Constant { level } => self.load_ascii_map(&level),
            PrefabMode::Sectional { section } => self.apply_sectional(&section),
            PrefabMode::RoomVaults => self.apply_room_vaults(),
        }
        self.take_snapshot();

        let mut start_idx;
        if self.starting_position.x == 0 {
            self.starting_position = Position {
                x: self.map.width / 2,
                y: self.map.height / 2,
            };
            start_idx = self
                .map
                .xy_idx(self.starting_position.x, self.starting_position.y);
            while self.map.tiles[start_idx] != TileType::Floor {
                self.starting_position.x -= 1;
                start_idx = self
                    .map
                    .xy_idx(self.starting_position.x, self.starting_position.y);
            }
            self.take_snapshot();

            let mut has_exit = false;
            self.map.tiles.iter().for_each(|t| {
                if *t == TileType::DownStairs {
                    has_exit = true;
                }
            });
            if !has_exit {
                start_idx = self
                    .map
                    .xy_idx(self.starting_position.x, self.starting_position.y);

                let exit_tile = remove_unreachable_areas_get_most_distant(&mut self.map, start_idx);
                self.take_snapshot();

                // Place the stairs
                self.map.tiles[exit_tile] = TileType::DownStairs;
                self.take_snapshot();
            }
        }
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

    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = self.map.xy_idx(x as i32, y as i32);
                        // NOTE: nasty casting.
                        self.char_to_map_tile(cell.ch as u8 as char, idx);
                    }
                }
            }
        }
    }

    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel) {
        // Start by converting to a vector, with newlines removed
        let string_vec =
            PrefabBuilder::read_ascii_to_vec(level.template, level.width, level.height);

        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = self.map.xy_idx(tx as i32, ty as i32);
                    self.char_to_map_tile(string_vec[i], idx);
                }
                i += 1;
            }
        }
    }

    fn char_to_map_tile(&mut self, ch: char, idx: usize) {
        let tile = match ch {
            ' ' | '@' | 'g' | '^' | '!' | '%' => Some(TileType::Floor),
            '#' => Some(TileType::Wall),
            '>' => Some(TileType::DownStairs),
            _ => None,
        };
        if let Some(tile) = tile {
            self.map.tiles[idx] = tile;
        }

        let spawn = match ch {
            'g' => Some("Goblin".to_string()),
            //'o' => Some(Monster::Orc), // TODO: Use enum for spawns. to_string() allocs to heap
            'o' => Some("Orc".to_string()),
            '^' => Some("Bear Trap".to_string()),
            '%' => Some("Rations".to_string()),
            '!' => Some("Health Potion".to_string()),
            _ => None,
        };
        if let Some(name) = spawn {
            self.spawn_list.push((idx, name));
        }

        if ch == '@' {
            // Player obviously stands on a floor tile
            let x = idx as i32 % self.map.width;
            let y = idx as i32 / self.map.width;
            self.starting_position = Position { x, y };
        }
    }

    fn apply_previous_iteration<F>(&mut self, mut filter: F)
    where
        F: FnMut(i32, i32, &(usize, String)) -> bool,
    {
        // Build the map
        let prev_builder = self.previous_builder.as_mut().unwrap();
        prev_builder.build_map();
        self.starting_position = prev_builder.get_starting_position();
        self.map = prev_builder.get_map();
        for e in prev_builder.get_spawn_list().iter() {
            let idx = e.0;
            let x = idx as i32 % self.map.width;
            let y = idx as i32 / self.map.width;
            if filter(x, y, e) {
                self.spawn_list.push((idx, e.1.to_string()))
            }
        }
        self.take_snapshot();
    }

    fn apply_sectional(&mut self, section: &prefab_sections::PrefabSection) {
        use prefab_sections::*;

        let string_vec =
            PrefabBuilder::read_ascii_to_vec(section.template, section.width, section.height);

        // Place the new section
        let chunk_x = match section.placement.0 {
            HorizontalPlacement::Left => 0,
            HorizontalPlacement::Center => (self.map.width / 2) - (section.width as i32 / 2),
            HorizontalPlacement::Right => (self.map.width - 1) - section.width as i32,
        };

        let chunk_y = match section.placement.1 {
            VerticalPlacement::Top => 0,
            VerticalPlacement::Center => (self.map.height / 2) - (section.height as i32 / 2),
            VerticalPlacement::Bottom => (self.map.height - 1) - section.height as i32,
        };

        // Build the map
        self.apply_previous_iteration(|x, y, _e| {
            x < chunk_x
                || x > (chunk_x + section.width as i32)
                || y < chunk_y
                || y > (chunk_y + section.height as i32)
        });

        let mut i = 0;
        for ty in 0..section.height {
            for tx in 0..section.width {
                if tx > 0
                    && tx < self.map.width as usize - 1
                    && ty < self.map.height as usize - 1
                    && ty > 0
                {
                    let idx = self.map.xy_idx(tx as i32 + chunk_x, ty as i32 + chunk_y);
                    self.char_to_map_tile(string_vec[i], idx);
                }
                i += 1;
            }
        }
        self.take_snapshot();
    }

    fn apply_room_vaults(&mut self) {
        use prefab_rooms::*;
        let mut rng = rltk::RandomNumberGenerator::new();

        // Apply the previous builder, and keep all entities it spawns (for now)
        self.apply_previous_iteration(|_x, _y, _e| true);

        // Vault spawns have 3/6 chance
        let vault_roll = rng.roll_dice(1, 6) + self.depth;
        if vault_roll < 4 {
            return;
        };

        // NOTE: place-holder, will be moved out of this function
        let master_vault_list = vec![TOTALLY_NOT_A_TRAP, SILLY_SMILE, CHECKERBOARD];

        // Filter vault list by current depth
        let mut possible_vaults: Vec<&PrefabRoom> = master_vault_list
            .iter()
            .filter(|v| self.depth >= v.first_depth && self.depth <= v.last_depth)
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

            for idx in 0..=self.map.tiles.len() - 1 {
                let x = (idx % self.map.width as usize) as i32;
                let y = (idx / self.map.width as usize) as i32;

                // Check map bounds
                if x > 1
                    && (x + vault.width as i32) < self.map.width - 2
                    && y > 1
                    && (y + vault.height as i32) < self.map.height - 2
                {
                    // TODO: REFACTOR
                    let mut possible = true;
                    'map: for ty in 0..vault.height as i32 {
                        for tx in 0..vault.width as i32 {
                            let t_idx = self.map.xy_idx(tx + x, ty + y);
                            if self.map.tiles[t_idx] != TileType::Floor || used_tiles.contains(&idx)
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
            self.spawn_list.retain(|e| {
                let idx = e.0 as i32;
                let (x, y) = self.map.idx_xy(idx as usize);
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
                    let idx = self.map.xy_idx(tx as i32 + pos.x, ty as i32 + pos.y);
                    self.char_to_map_tile(string_vec[i], idx);
                    used_tiles.insert(idx);
                    i += 1;
                }
            }
            self.take_snapshot();
            possible_vaults.remove(vault_index);
        }
    }
}
