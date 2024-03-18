use crate::{map, TileType};

use super::{BuilderMap, MetaMapBuilder};

#[allow(dead_code)]
pub enum XEnd {
    Left,
    Center,
    Right,
}

#[allow(dead_code)]
pub enum YEnd {
    Top,
    Center,
    Bottom,
}

pub struct AreaEndingPosition {
    x: XEnd,
    y: YEnd,
}

impl MetaMapBuilder for AreaEndingPosition {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl AreaEndingPosition {
    pub fn new(x: XEnd, y: YEnd) -> Box<AreaEndingPosition> {
        Box::new(AreaEndingPosition { x, y })
    }

    fn build(&mut self, _rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        let seed_x = match self.x {
            XEnd::Left => 1,
            XEnd::Center => build_data.map.width / 2,
            XEnd::Right => build_data.map.width - 2,
        };

        let seed_y = match self.y {
            YEnd::Top => 1,
            YEnd::Center => build_data.map.height / 2,
            YEnd::Bottom => build_data.map.height - 2,
        };

        let mut available_floors = Vec::new();
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

        build_data.map.tiles[available_floors[0].0] = TileType::DownStairs;
    }
}
