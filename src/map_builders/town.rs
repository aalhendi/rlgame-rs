use super::{BuilderChain, BuilderMap, InitialMapBuilder};
use crate::{components::Position, map::TileType};
use std::collections::HashSet;

pub fn town_builder(
    new_depth: i32,
    _rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth, width, height);
    builder.start_with(TownBuilder::new());
    builder
}

enum BuildingTag {
    Pub,
    Temple,
    Blacksmith,
    Clothier,
    Alchemist,
    PlayerHouse,
    Hovel,
    Abandoned,
    Unassigned,
}
struct RoomEdges {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

pub struct TownBuilder {}

impl InitialMapBuilder for TownBuilder {
    fn build_map(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut super::BuilderMap,
    ) {
        self.build_rooms(rng, build_data);
    }
}

impl TownBuilder {
    pub fn new() -> Box<TownBuilder> {
        Box::new(TownBuilder {})
    }

    pub fn build_rooms(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        self.grass_layer(build_data);
        self.water_and_piers(rng, build_data);
        let (mut available_building_tiles, wall_gap_y) = self.town_walls(rng, build_data);
        let mut buildings = self.buildings(rng, build_data, &mut available_building_tiles);
        let doors = self.add_doors(rng, build_data, &mut buildings, wall_gap_y);
        self.add_paths(build_data, &doors);

        // Set visible tiles for mapgen visualizer
        build_data.map.visible_tiles.iter_mut().for_each(|t| {
            *t = true;
        });
        build_data.take_snapshot();

        // Set Exit
        let exit_idx = build_data.map.xy_idx(build_data.width - 5, wall_gap_y);
        build_data.map.tiles[exit_idx] = TileType::DownStairs;

        let building_sizes = self.sort_buildings(&buildings);

        // Also sets player spawn
        self.building_factory(rng, build_data, &buildings, &building_sizes);
    }

    fn sort_buildings(&mut self, buildings: &[RoomEdges]) -> Vec<(usize, i32, BuildingTag)> {
        let mut b_sizes: Vec<(usize, i32, BuildingTag)> = buildings
            .iter()
            .enumerate()
            .map(|(i, b)| (i, b.w * b.h, BuildingTag::Unassigned))
            .collect();
        b_sizes.sort_by(|a, b| a.1.cmp(&b.1));
        b_sizes[0].2 = BuildingTag::Pub;
        b_sizes[1].2 = BuildingTag::Temple;
        b_sizes[2].2 = BuildingTag::Blacksmith;
        b_sizes[3].2 = BuildingTag::Clothier;
        b_sizes[4].2 = BuildingTag::Alchemist;
        b_sizes[5].2 = BuildingTag::PlayerHouse;
        for b in b_sizes.iter_mut().skip(6) {
            b.2 = BuildingTag::Hovel;
        }
        b_sizes.last_mut().unwrap().2 = BuildingTag::Abandoned;

        b_sizes
    }

    /// Sets all tiles as ``TileType::Grass`` and takes snapshot
    fn grass_layer(&mut self, build_data: &mut BuilderMap) {
        build_data.map.tiles.iter_mut().for_each(|t| {
            *t = TileType::Grass;
        });
        build_data.take_snapshot();
    }

    fn water_and_piers(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        // Random float between 0.0 and 1.0
        let mut n = (rng.roll_dice(1, 65535) as f32) / 65535_f32;
        let mut water_width = Vec::new();

        // Generate water row iteratively
        for y in 0..build_data.height {
            let n_water = (f32::sin(n) * 10_f32) as i32 + 14 + rng.roll_dice(1, 6);
            water_width.push(n_water);
            n += 0.1;
            for x in 0..n_water {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::DeepWater;
            }
            for x in n_water..n_water + 3 {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::ShallowWater;
            }
        }
        build_data.take_snapshot();

        // Add piers (n = 10~14)
        for _ in 0..rng.roll_dice(1, 4) + 6 {
            let y = rng.roll_dice(1, build_data.height) - 1;
            for x in 2 + rng.roll_dice(1, 6)..water_width[y as usize] + 4 {
                let idx = build_data.map.xy_idx(x, y);
                build_data.map.tiles[idx] = TileType::WoodFloor;
            }
        }
        build_data.take_snapshot();
    }

    /// Generates and sets town border walls and a horizontal road.
    fn town_walls(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) -> (HashSet<usize>, i32) {
        let mut available_building_tiles = HashSet::new();
        // Height of road going through town horizontally
        let wall_gap_y = rng.roll_dice(1, build_data.height - 9) + 5;
        for y in 1..build_data.height - 2 {
            if !(y > wall_gap_y - 4 && y < wall_gap_y + 4) {
                let idx = build_data.map.xy_idx(30, y);
                let idx_right = build_data.map.xy_idx(build_data.width - 2, y);
                build_data.map.tiles[idx] = TileType::Wall;
                build_data.map.tiles[idx - 1] = TileType::Floor;
                build_data.map.tiles[idx_right] = TileType::Wall;
                for x in 31..build_data.width - 2 {
                    let gravel_idx = build_data.map.xy_idx(x, y);
                    build_data.map.tiles[gravel_idx] = TileType::Gravel;
                    if y > 2 && y < build_data.height - 1 {
                        available_building_tiles.insert(gravel_idx);
                    }
                }
            } else {
                for x in 30..build_data.width {
                    let road_idx = build_data.map.xy_idx(x, y);
                    build_data.map.tiles[road_idx] = TileType::Road;
                }
            }
        }
        build_data.take_snapshot();

        for x in 30..build_data.width - 1 {
            // TODO: Add a set_tile fn to map...
            let idx_top = build_data.map.xy_idx(x, 1);
            build_data.map.tiles[idx_top] = TileType::Wall;
            let idx_bot = build_data.map.xy_idx(x, build_data.height - 2);
            build_data.map.tiles[idx_bot] = TileType::Wall;
        }
        build_data.take_snapshot();

        (available_building_tiles, wall_gap_y)
    }

    /// Generates and sets 12 buildings (rects) within town walls
    fn buildings(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
        available_building_tiles: &mut HashSet<usize>,
    ) -> Vec<RoomEdges> {
        let w = build_data.width as usize;
        let mut buildings = Vec::new();
        let mut n_buildings = 0;
        while n_buildings < 12 {
            let bx = rng.roll_dice(1, build_data.map.width - 32) + 30;
            let by = rng.roll_dice(1, build_data.map.height) - 2;
            let bw = rng.roll_dice(1, 8) + 4;
            let bh = rng.roll_dice(1, 8) + 4;
            let mut possible = true;
            for y in by..by + bh {
                for x in bx..bx + bw {
                    if x < 0 || x > build_data.width - 1 || y < 0 || y > build_data.height - 1 {
                        possible = false;
                    } else {
                        let idx = build_data.map.xy_idx(x, y);
                        if !available_building_tiles.contains(&idx) {
                            possible = false;
                        }
                    }
                }
            }
            if possible {
                n_buildings += 1;
                buildings.push(RoomEdges {
                    x: bx,
                    y: by,
                    w: bw,
                    h: bh,
                });

                for y in by..by + bh {
                    for x in bx..bx + bw {
                        let idx = build_data.map.xy_idx(x, y);
                        build_data.map.tiles[idx] = TileType::WoodFloor;
                        available_building_tiles.remove(&idx);
                        available_building_tiles.remove(&(idx + 1));
                        available_building_tiles.remove(&(idx + w));
                        available_building_tiles.remove(&(idx - 1));
                        available_building_tiles.remove(&(idx - w));
                    }
                }
                build_data.take_snapshot();
            }
        }
        // Outline buildings
        let mut mapclone = build_data.map.clone();
        for y in 2..build_data.height - 2 {
            for x in 32..build_data.width - 2 {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::WoodFloor {
                    let mut neighbors = 0;
                    neighbors += (build_data.map.tiles[idx - 1] != TileType::WoodFloor) as i32;
                    neighbors += (build_data.map.tiles[idx + 1] != TileType::WoodFloor) as i32;
                    neighbors += (build_data.map.tiles[idx - w] != TileType::WoodFloor) as i32;
                    neighbors += (build_data.map.tiles[idx + w] != TileType::WoodFloor) as i32;
                    if neighbors > 0 {
                        mapclone.tiles[idx] = TileType::Wall;
                    }
                }
            }
        }
        build_data.map = mapclone;
        build_data.take_snapshot();
        buildings
    }

    fn add_doors(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &mut [RoomEdges],
        wall_gap_y: i32,
    ) -> Vec<usize> {
        let mut doors = Vec::new();
        for building in buildings.iter() {
            let door_x = building.x + 1 + rng.roll_dice(1, building.w - 3);
            let center_y = building.y + (building.h / 2);
            let idx = if center_y > wall_gap_y {
                // Door on the north wall
                build_data.map.xy_idx(door_x, building.y)
            } else {
                build_data.map.xy_idx(door_x, building.y + building.h - 1)
            };
            build_data.map.tiles[idx] = TileType::Floor;
            build_data.spawn_list.push((idx, "Door".to_string()));
            doors.push(idx);
        }
        build_data.take_snapshot();
        doors
    }

    fn add_paths(&mut self, build_data: &mut BuilderMap, doors: &[usize]) {
        let mut roads = Vec::new();
        for y in 0..build_data.height {
            for x in 0..build_data.width {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::Road {
                    roads.push(idx);
                }
            }
        }

        build_data.map.populate_blocked();
        for door_idx in doors.iter() {
            let mut nearest_roads: Vec<(usize, f32)> = Vec::new();
            let (x, y) = build_data.map.idx_xy(*door_idx);
            let door_pt = rltk::Point::new(x, y);
            for r in roads.iter() {
                let (x, y) = build_data.map.idx_xy(*r);
                nearest_roads.push((
                    *r,
                    rltk::DistanceAlg::PythagorasSquared
                        .distance2d(door_pt, rltk::Point::new(x, y)),
                ));
            }
            nearest_roads
                .sort_by(|(_a_idx, a_dist), (_b_idx, b_dist)| a_dist.partial_cmp(b_dist).unwrap());

            let (dest, _dist) = nearest_roads[0];
            let path = rltk::a_star_search(*door_idx, dest, &build_data.map);
            if path.success {
                for step in path.steps.iter() {
                    let idx = *step;
                    build_data.map.tiles[idx] = TileType::Road;
                    roads.push(idx);
                }
            }
            build_data.take_snapshot();
        }
    }

    fn building_factory(
        &mut self,
        rng: &mut rltk::RandomNumberGenerator,
        build_data: &mut BuilderMap,
        buildings: &[RoomEdges],
        buildings_data: &[(usize, i32, BuildingTag)],
    ) {
        for (i, b) in buildings.iter().enumerate() {
            match &buildings_data[i].2 {
                BuildingTag::Pub => self.build_pub(b, build_data, rng),
                BuildingTag::Temple => self.build_temple(b, build_data, rng),
                BuildingTag::Blacksmith => self.build_smith(b, build_data, rng),
                BuildingTag::Clothier => self.build_clothier(b, build_data, rng),
                BuildingTag::Alchemist => self.build_alchemist(b, build_data, rng),
                BuildingTag::PlayerHouse => self.build_player_house(b, build_data, rng),
                BuildingTag::Hovel => self.build_hovel(b, build_data, rng),
                BuildingTag::Abandoned => self.build_abandoned_house(b, build_data, rng),
                BuildingTag::Unassigned => panic!("Unassigned building"),
            }
        }
    }

    fn build_pub(
        &mut self,
        building: &RoomEdges,
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        // Place the player
        let cx = building.x + (building.w / 2);
        let cy = building.y + (building.h / 2);
        build_data.starting_position = Some(Position { x: cx, y: cy });
        let player_idx = build_data.map.xy_idx(cx, cy);

        // Place others
        let to_place = vec![
            "Chair",
            "Table",
            "Chair",
            "Table",
            "Keg",
            "Patron",
            "Patron",
            "Shady Salesman",
            "Barkeep",
        ];
        self.random_building_spawn(building, build_data, player_idx, rng, to_place);
    }

    fn build_temple(
        &mut self,
        building: &RoomEdges,
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        let to_place = vec![
            "Candle",
            "Candle",
            "Chair",
            "Chair",
            "Parishioner",
            "Parishioner",
            "Priest",
        ];
        self.random_building_spawn(building, build_data, 0, rng, to_place);
    }

    fn build_smith(
        &mut self,
        building: &RoomEdges,
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        let to_place: Vec<&str> = vec![
            "Armor Stand",
            "Weapon Rack",
            "Water Trough",
            "Anvil",
            "Blacksmith",
        ];
        self.random_building_spawn(building, build_data, 0, rng, to_place);
    }

    fn build_clothier(
        &mut self,
        building: &RoomEdges,
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        let to_place: Vec<&str> = vec!["Hide Rack", "Loom", "Table", "Cabinet", "Clothier"];

        self.random_building_spawn(building, build_data, 0, rng, to_place);
    }

    fn build_abandoned_house(
        &mut self,
        building: &RoomEdges,
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        for y in building.y..building.y + building.h {
            for x in building.x..building.x + building.w {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::WoodFloor && rng.roll_dice(1, 2) == 1 {
                    build_data.spawn_list.push((idx, "Rat".to_string()));
                }
            }
        }
    }

    fn build_alchemist(
        &mut self,
        building: &RoomEdges,
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        let to_place: Vec<&str> =
            vec!["Table", "Chair", "Dead Thing", "Chemistry Set", "Alchemist"];

        self.random_building_spawn(building, build_data, 0, rng, to_place);
    }

    fn build_player_house(
        &mut self,
        building: &RoomEdges,
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        let to_place: Vec<&str> = vec!["Table", "Chair", "Cabinet", "Bed", "Mom"];

        self.random_building_spawn(building, build_data, 0, rng, to_place);
    }

    fn build_hovel(
        &mut self,
        building: &RoomEdges,
        build_data: &mut BuilderMap,
        rng: &mut rltk::RandomNumberGenerator,
    ) {
        let to_place: Vec<&str> = vec!["Table", "Chair", "Bed", "Peasant"];

        self.random_building_spawn(building, build_data, 0, rng, to_place);
    }

    fn random_building_spawn(
        &mut self,
        building: &RoomEdges,
        build_data: &mut BuilderMap,
        player_idx: usize,
        rng: &mut rltk::RandomNumberGenerator,
        mut to_place: Vec<&str>,
    ) {
        for y in building.y..building.y + building.h {
            for x in building.x..building.x + building.w {
                let idx = build_data.map.xy_idx(x, y);
                if build_data.map.tiles[idx] == TileType::WoodFloor
                    && idx != player_idx
                    && rng.roll_dice(1, 3) == 1
                {
                    if let Some(e_tag) = to_place.pop() {
                        build_data.spawn_list.push((idx, e_tag.to_string()));
                    }
                }
            }
        }
    }
}
