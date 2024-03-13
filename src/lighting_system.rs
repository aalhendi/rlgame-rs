use super::{LightSource, Map, Position, Viewshed};
use rltk::RGB;
use specs::{Join, ReadStorage, System, WriteExpect};

pub struct LightingSystem;

impl<'a> System<'a> for LightingSystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadStorage<'a, Viewshed>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, LightSource>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, viewsheds, positions, lightings) = data;

        if map.outdoors {
            return;
        }

        for l in map.light_level_tiles.iter_mut() {
            *l = RGB::named(rltk::BLACK);
        }

        for (viewshed, pos, light) in (&viewsheds, &positions, &lightings).join() {
            let light_point = rltk::Point::new(pos.x, pos.y);
            let range_f = light.range as f32;
            for t in viewshed.visible_tiles.iter() {
                if t.x > 0 && t.x < map.width && t.y > 0 && t.y < map.height {
                    let idx = map.xy_idx(t.x, t.y);
                    let distance = rltk::DistanceAlg::Pythagoras.distance2d(light_point, *t);
                    let intensity = (range_f - distance) / range_f;

                    map.light_level_tiles[idx] =
                        map.light_level_tiles[idx] + (light.color * intensity);
                }
            }
        }
    }
}
