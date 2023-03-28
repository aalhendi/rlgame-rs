use super::{
    common::apply_horizontal_tunnel, common::apply_room_to_map, common::apply_vertical_tunnel,
    BuilderMap, InitialMapBuilder,
};
use crate::Rect;
use rltk::RandomNumberGenerator;

pub struct SimpleMapBuilder {}

impl InitialMapBuilder for SimpleMapBuilder {
    fn build_map(&mut self, rng: &mut rltk::RandomNumberGenerator, build_data: &mut BuilderMap) {
        self.rooms_and_corridors(rng, build_data);
    }
}

impl SimpleMapBuilder {
    pub fn new() -> Box<SimpleMapBuilder> {
        Box::new(SimpleMapBuilder {})
    }

    /// Makes a new map using the algorithm from <http://rogueliketutorials.com/tutorials/tcod/part-3/>
    /// Returns map with random rooms and corridors to join them.
    pub fn rooms_and_corridors(
        &mut self,
        rng: &mut RandomNumberGenerator,
        build_data: &mut BuilderMap,
    ) {
        const MAX_ROOMS: i32 = 30;
        const MIN_SIZE: i32 = 6;
        const MAX_SIZE: i32 = 10;
        let mut rooms: Vec<Rect> = Vec::new();

        for _ in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, build_data.map.width - 1 - w) - 1;
            let y = rng.roll_dice(1, build_data.map.height - 1 - h) - 1;

            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in rooms.iter() {
                if new_room.intersects(other_room) {
                    ok = false;
                }
            }
            if ok {
                apply_room_to_map(&mut build_data.map, &new_room);
                build_data.take_snapshot();

                if !rooms.is_empty() {
                    let new_center = new_room.center();
                    let old_center = rooms[rooms.len() - 1].center();
                    if rng.range(0, 2) == 1 {
                        apply_horizontal_tunnel(
                            &mut build_data.map,
                            old_center.x,
                            new_center.x,
                            old_center.y,
                        );
                        apply_vertical_tunnel(
                            &mut build_data.map,
                            old_center.y,
                            new_center.y,
                            new_center.x,
                        );
                    } else {
                        apply_vertical_tunnel(
                            &mut build_data.map,
                            old_center.y,
                            new_center.y,
                            new_center.x,
                        );
                        apply_horizontal_tunnel(
                            &mut build_data.map,
                            old_center.x,
                            new_center.x,
                            old_center.y,
                        );
                    }
                }

                rooms.push(new_room);
                build_data.take_snapshot();
            }
        }
        build_data.rooms = Some(rooms);
    }
}
