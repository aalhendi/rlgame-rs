use rltk::RandomNumberGenerator;

use crate::{map, TileType};

use super::{BuilderMap, MetaMapBuilder};

pub struct YellowBrickRoad {}

impl MetaMapBuilder for YellowBrickRoad {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl YellowBrickRoad {
    pub fn new() -> Box<YellowBrickRoad> {
        Box::new(YellowBrickRoad {})
    }

    // TODO(aalhendi): return idx?
    fn find_exit(&self, build_data: &mut BuilderMap, seed_x: i32, seed_y: i32) -> (i32, i32) {
        let mut available_floors: Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if map::tile_walkable(*tiletype) {
                let (x, y) = build_data.map.idx_xy(idx);
                available_floors.push((
                    idx,
                    rltk::DistanceAlg::PythagorasSquared
                        .distance2d(rltk::Point::new(x, y), rltk::Point::new(seed_x, seed_y)),
                ));
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors
            .sort_by(|(_a_idx, a_dist), (_b_idx, b_dist)| a_dist.partial_cmp(b_dist).unwrap());

        let idx = available_floors[0].0;
        build_data.map.idx_xy(idx) // return end_x, end_y
    }

    // TODO(aalhendi): Should this be refactored into Map?
    fn paint_road(&self, build_data: &mut BuilderMap, x: i32, y: i32) {
        if x < 1 || x > build_data.map.width - 2 || y < 1 || y > build_data.map.height - 2 {
            return;
        }
        let idx = build_data.map.xy_idx(x, y);
        if build_data.map.tiles[idx] != TileType::DownStairs {
            build_data.map.tiles[idx] = TileType::Road;
        }
    }

    fn build(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let starting_pos = *build_data.starting_position.as_ref().unwrap();
        let start_idx = build_data.map.xy_idx(starting_pos.x, starting_pos.y);

        // Calculate road path
        let (end_x, end_y) = self.find_exit(
            build_data,
            build_data.map.width - 2,
            build_data.map.height / 2,
        );
        let end_idx = build_data.map.xy_idx(end_x, end_y);
        build_data.map.populate_blocked();
        let path = rltk::a_star_search(start_idx, end_idx, &build_data.map);

        // Paint 3x3 road
        for idx in path.steps {
            let (x, y) = build_data.map.idx_xy(idx);

            self.paint_road(build_data, x, y);
            self.paint_road(build_data, x - 1, y);
            self.paint_road(build_data, x + 1, y);
            self.paint_road(build_data, x, y - 1);
            self.paint_road(build_data, x, y + 1);
        }
        build_data.take_snapshot();

        // Calculate exit path
        let exit_dir = rng.roll_dice(1, 2);
        let (seed_x, seed_y, stream_startx, stream_starty) = {
            let w = build_data.map.width;
            let h = build_data.map.height;
            match exit_dir {
                // North-East
                1 => (w - 1, 1, 0, h - 1),
                // South-East
                _ => (w - 1, h - 1, 1, h - 1),
            }
        };

        let (stairs_x, stairs_y) = self.find_exit(build_data, seed_x, seed_y);
        let stairs_idx = build_data.map.xy_idx(stairs_x, stairs_y);
        build_data.take_snapshot();

        let (stream_x, stream_y) = self.find_exit(build_data, stream_startx, stream_starty);
        let stream_idx = build_data.map.xy_idx(stream_x, stream_y);
        let stream = rltk::a_star_search(stairs_idx, stream_idx, &build_data.map);

        // Paint exit path (stream of water)
        for tile in stream.steps.iter() {
            if build_data.map.tiles[*tile] == TileType::Floor {
                build_data.map.tiles[*tile] = TileType::ShallowWater;
            }
        }

        // Place exit
        build_data.map.tiles[stairs_idx] = TileType::DownStairs;
        build_data.take_snapshot();
    }
}
