use crate::map::TileType;

use super::{BuilderMap, MetaMapBuilder, Position};
use rltk::RandomNumberGenerator;

pub enum XStart {
    Left,
    Center,
    Right,
}

pub enum YStart {
    Top,
    Center,
    Bottom,
}

pub struct AreaStartingPosition {
    x: XStart,
    y: YStart,
}

impl MetaMapBuilder for AreaStartingPosition {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.build(rng, build_data);
    }
}

impl AreaStartingPosition {
    pub fn new(x: XStart, y: YStart) -> Box<AreaStartingPosition> {
        Box::new(AreaStartingPosition { x, y })
    }

    fn build(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let seed_x = match self.x {
            XStart::Left => 1,
            XStart::Center => build_data.map.width / 2,
            XStart::Right => build_data.map.width - 2,
        };

        let seed_y = match self.y {
            YStart::Top => 1,
            YStart::Center => build_data.map.height / 2,
            YStart::Bottom => build_data.map.height - 2,
        };

        let mut available_floors: Vec<(usize, f32)> = Vec::new();
        for (idx, tiletype) in build_data.map.tiles.iter().enumerate() {
            if *tiletype == TileType::Floor {
                available_floors.push((
                    idx,
                    rltk::DistanceAlg::PythagorasSquared.distance2d(
                        rltk::Point::new(
                            idx as i32 % build_data.map.width,
                            idx as i32 / build_data.map.width,
                        ),
                        rltk::Point::new(seed_x, seed_y),
                    ),
                ));
            }
        }
        if available_floors.is_empty() {
            panic!("No valid floors to start on");
        }

        available_floors
            .sort_by(|(_a_idx, a_dist), (_b_idx, b_dist)| a_dist.partial_cmp(b_dist).unwrap());

        let (start_x, start_y) = build_data.map.idx_xy(available_floors[0].0);

        build_data.starting_position = Some(Position {
            x: start_x,
            y: start_y,
        });
    }
}
