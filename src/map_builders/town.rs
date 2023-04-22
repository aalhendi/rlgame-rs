use super::{
    area_starting_points::AreaStartingPosition, distant_exit::DistantExit, random_start_position,
    BuilderChain, BuilderMap, InitialMapBuilder,
};
use crate::map::TileType;
use std::collections::HashSet;

pub fn town_builder(
    new_depth: i32,
    rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut builder = BuilderChain::new(new_depth, width, height);
    builder.start_with(TownBuilder::new());
    let (start_x, start_y) = random_start_position(rng);
    builder.with(AreaStartingPosition::new(start_x, start_y));
    builder.with(DistantExit::new());
    builder
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
        // let (mut available_building_tiles, wall_gap_y) = self.town_walls(rng, build_data);
        self.town_walls(rng, build_data);

        // Set visible tiles for mapgen visualizer
        build_data.map.visible_tiles.iter_mut().for_each(|t| {
            *t = true;
        });
        build_data.take_snapshot();
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
}
