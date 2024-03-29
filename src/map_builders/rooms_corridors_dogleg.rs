use super::{
    common::apply_horizontal_tunnel, common::apply_vertical_tunnel, BuilderMap, MetaMapBuilder,
    Rect,
};
use rltk::RandomNumberGenerator;

pub struct DoglegCorridors {}

impl MetaMapBuilder for DoglegCorridors {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.corridors(rng, build_data);
    }
}

impl DoglegCorridors {
    pub fn new() -> Box<DoglegCorridors> {
        Box::new(DoglegCorridors {})
    }

    fn corridors(&mut self, rng: &mut RandomNumberGenerator, build_data: &mut BuilderMap) {
        let rooms: Vec<Rect>;
        if let Some(rooms_builder) = &build_data.rooms {
            rooms = rooms_builder.clone();
        } else {
            panic!("Dogleg Corridors require a builder with room structures");
        }

        let mut corridors = Vec::new();
        for (i, room) in rooms.iter().enumerate() {
            if i == 0 {
                continue;
            }
            let new = room.center();
            let prev = rooms[i - 1].center();
            if rng.range(0, 2) == 1 {
                let mut c1 = apply_horizontal_tunnel(&mut build_data.map, prev.x, new.x, prev.y);
                let mut c2 = apply_vertical_tunnel(&mut build_data.map, prev.y, new.y, new.x);
                c1.append(&mut c2);
                corridors.push(c1);
            } else {
                let mut c1 = apply_vertical_tunnel(&mut build_data.map, prev.y, new.y, prev.x);
                let mut c2 = apply_horizontal_tunnel(&mut build_data.map, prev.x, new.x, new.y);
                c1.append(&mut c2);
                corridors.push(c1);
            }
            build_data.take_snapshot();
        }
        build_data.corridors = Some(corridors);
    }
}
