use super::{common::draw_corridor, BuilderMap, MetaMapBuilder};
use rltk::RandomNumberGenerator;
use std::collections::HashSet;

pub struct NearestCorridors {}

impl MetaMapBuilder for NearestCorridors {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

impl NearestCorridors {
    pub fn new() -> Box<NearestCorridors> {
        Box::new(NearestCorridors {})
    }

    fn corridors(&mut self, _rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms = if let Some(rooms_builder) = &build_data.rooms {
            rooms_builder.clone()
        } else {
            panic!("Nearest Corridors require a builder with room structures");
        };

        let mut connected: HashSet<usize> = HashSet::new();
        let mut corridors = Vec::new();
        for (i, room) in rooms.iter().enumerate() {
            let mut room_distance: Vec<(usize, f32)> = Vec::new();
            let room_center = room.center();
            let room_center_pt = rltk::Point::new(room_center.x, room_center.y);
            for (j, other_room) in rooms.iter().enumerate() {
                if i != j && !connected.contains(&j) {
                    let other_center = other_room.center();
                    let other_center_pt = rltk::Point::new(other_center.x, other_center.y);
                    let distance =
                        rltk::DistanceAlg::Pythagoras.distance2d(room_center_pt, other_center_pt);
                    room_distance.push((j, distance));
                }
            }

            if !room_distance.is_empty() {
                room_distance.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                let dest_center = rooms[room_distance[0].0].center();
                corridors.push(draw_corridor(
                    &mut build_data.map,
                    room_center.x,
                    room_center.y,
                    dest_center.x,
                    dest_center.y,
                ));
                connected.insert(i);
                build_data.take_snapshot();
            }
        }
        build_data.corridors = Some(corridors);
    }
}
