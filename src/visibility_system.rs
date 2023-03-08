use super::{gamelog::Gamelog, Hidden, Map, Name, Player, Position, Viewshed};
use rltk::{field_of_view, Point};
use specs::prelude::*;

pub struct VisibilitySystem;

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, Position>,
        ReadStorage<'a, Player>,
        WriteStorage<'a, Hidden>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        WriteExpect<'a, Gamelog>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut map, entities, mut viewshed, pos, player, mut hidden, mut rng, mut log, names) =
            data;

        for (ent, viewshed, pos) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles =
                    field_of_view(Point::new(pos.x, pos.y), viewshed.range, &*map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                // If player, reveal visible tiles
                if player.get(ent).is_some() {
                    for t in map.visible_tiles.iter_mut() {
                        *t = false
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;

                        // Chance to reveal hidden things
                        for e in map.tile_content[idx].iter() {
                            if hidden.get(*e).is_some() && rng.roll_dice(1, 24) == 1 {
                                if let Some(name) = names.get(*e) {
                                    log.entries
                                        .push(format!("You spotted a {name}.", name = &name.name));
                                }
                                hidden.remove(*e);
                            }
                        }
                    }
                }
                viewshed.dirty = false;
            }
        }
    }
}
