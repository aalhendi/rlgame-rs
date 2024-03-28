use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteStorage};

use crate::{
    raws::{faction_structs::Reaction, rawsmaster::faction_reaction, RAWS},
    spatial, Faction, Map, MyTurn, Position, TileSize, WantsToMelee,
};

pub struct AdjacentAI;

impl<'a> System<'a> for AdjacentAI {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, WantsToMelee>,
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, TileSize>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, factions, positions, map, mut want_melee, entities, player, sizes) = data;

        let mut turn_done = Vec::new();
        for (entity, _turn, my_fac, pos) in (&entities, &turns, &factions, &positions).join() {
            if entity == *player {
                continue;
            }
            let mut reactions = Vec::new();
            let idx = map.xy_idx(pos.x, pos.y) as i32;
            let w = map.width;
            let h = map.height;

            // If multitile
            if let Some(size) = sizes.get(entity) {
                use crate::rect::Rect;
                let mob_rect = Rect::new(pos.x, pos.y, size.x, size.y).get_all_tiles();
                let parent_rect = Rect::new(pos.x - 1, pos.y - 1, size.x + 2, size.y + 2);
                parent_rect
                    .get_all_tiles()
                    .iter()
                    .filter(|t| !mob_rect.contains(t))
                    .for_each(|t| {
                        if t.0 > 0 && t.0 < w - 1 && t.1 > 0 && t.1 < h - 1 {
                            let tgt_idx = map.xy_idx(t.0, t.1) as i32;
                            evaluate(tgt_idx, &factions, &my_fac.name, &mut reactions);
                        }
                    });
                continue;
            }

            // Add possible reactions to adjacents for each direction
            if pos.x > 0 {
                evaluate(idx - 1, &factions, &my_fac.name, &mut reactions);
            }
            if pos.x < w - 1 {
                evaluate(idx + 1, &factions, &my_fac.name, &mut reactions);
            }
            if pos.y > 0 {
                evaluate(idx - w, &factions, &my_fac.name, &mut reactions);
            }
            if pos.y < h - 1 {
                evaluate(idx + w, &factions, &my_fac.name, &mut reactions);
            }
            if pos.y > 0 && pos.x > 0 {
                evaluate(idx - w - 1, &factions, &my_fac.name, &mut reactions);
            }
            if pos.y > 0 && pos.x < w - 1 {
                evaluate(idx - w + 1, &factions, &my_fac.name, &mut reactions);
            }
            if pos.y < h - 1 && pos.x > 0 {
                evaluate(idx + w - 1, &factions, &my_fac.name, &mut reactions);
            }
            if pos.y < h - 1 && pos.x < w - 1 {
                evaluate(idx + w + 1, &factions, &my_fac.name, &mut reactions);
            }

            let mut done = false;
            for (target, reaction) in reactions.iter() {
                if let Reaction::Attack = reaction {
                    want_melee
                        .insert(entity, WantsToMelee { target: *target })
                        .expect("Error inserting melee");
                    done = true;
                }
            }

            if done {
                turn_done.push(entity);
            }
        }

        // Remove turn marker for those that are done
        for done in turn_done.iter() {
            turns.remove(*done);
        }
    }
}

fn evaluate(
    idx: i32,
    factions: &ReadStorage<Faction>,
    my_fac: &str,
    reactions: &mut Vec<(Entity, Reaction)>,
) {
    spatial::for_each_tile_content(idx as usize, |other_entity| {
        if let Some(faction) = factions.get(other_entity) {
            reactions.push((
                other_entity,
                faction_reaction(my_fac, &faction.name, &RAWS.lock().unwrap()),
            ));
        }
    });
}
