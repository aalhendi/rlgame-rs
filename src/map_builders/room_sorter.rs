use crate::rect::Rect;

use super::{BuilderMap, MetaMapBuilder};
use rltk::RandomNumberGenerator;

pub enum RoomSort {
    Leftmost,
    Rightmost,
    Topmost,
    Bottommost,
    Central,
}

pub struct RoomSorter {
    sort_by: RoomSort,
}

impl MetaMapBuilder for RoomSorter {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.sorter(rng, build_data);
    }
}

impl RoomSorter {
    pub fn new(sort_by: RoomSort) -> Box<RoomSorter> {
        Box::new(RoomSorter { sort_by })
    }

    fn sorter(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms = build_data.rooms.as_mut().unwrap();
        match self.sort_by {
            RoomSort::Leftmost => rooms.sort_by(|a, b| a.x1.cmp(&b.x1)),
            RoomSort::Rightmost => rooms.sort_by(|a, b| b.x2.cmp(&a.x2)),
            RoomSort::Topmost => rooms.sort_by(|a, b| a.y1.cmp(&b.y1)),
            RoomSort::Bottommost => rooms.sort_by(|a, b| b.y2.cmp(&a.y2)),
            RoomSort::Central => {
                let map_center =
                    rltk::Point::new(build_data.map.width / 2, build_data.map.height / 2);
                let center_sort = |a: &Rect, b: &Rect| {
                    let a_center = a.center();
                    let a_center_pt = rltk::Point::new(a_center.x, a_center.y);
                    let b_center = b.center();
                    let b_center_pt = rltk::Point::new(b_center.x, b_center.y);
                    let distance_a =
                        rltk::DistanceAlg::Pythagoras.distance2d(a_center_pt, map_center);
                    let distance_b =
                        rltk::DistanceAlg::Pythagoras.distance2d(b_center_pt, map_center);
                    distance_a.partial_cmp(&distance_b).unwrap()
                };
                rooms.sort_by(center_sort)
            }
        }
    }
}
