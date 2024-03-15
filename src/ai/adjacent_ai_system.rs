use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteStorage};

use crate::{
    raws::{faction_structs::Reaction, rawsmaster::faction_reaction, RAWS},
    Faction, Map, MyTurn, Position, WantsToMelee,
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
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut turns, factions, positions, map, mut want_melee, entities, player) = data;

        let mut turn_done = Vec::new();
        for (entity, _turn, my_fac, pos) in (&entities, &turns, &factions, &positions).join() {
            if entity == *player {
                continue;
            }
            let mut reactions = Vec::new();
            let idx = map.xy_idx(pos.x, pos.y) as i32;
            let w = map.width;
            let h = map.height;
            // Add possible reactions to adjacents for each direction
            if pos.x > 0 {
                evaluate(idx - 1, &map, &factions, &my_fac.name, &mut reactions);
            }
            if pos.x < w - 1 {
                evaluate(idx + 1, &map, &factions, &my_fac.name, &mut reactions);
            }
            if pos.y > 0 {
                evaluate(idx - w, &map, &factions, &my_fac.name, &mut reactions);
            }
            if pos.y < h - 1 {
                evaluate(idx + w, &map, &factions, &my_fac.name, &mut reactions);
            }
            if pos.y > 0 && pos.x > 0 {
                evaluate(idx - w - 1, &map, &factions, &my_fac.name, &mut reactions);
            }
            if pos.y > 0 && pos.x < w - 1 {
                evaluate(idx - w + 1, &map, &factions, &my_fac.name, &mut reactions);
            }
            if pos.y < h - 1 && pos.x > 0 {
                evaluate(idx + w - 1, &map, &factions, &my_fac.name, &mut reactions);
            }
            if pos.y < h - 1 && pos.x < w - 1 {
                evaluate(idx + w + 1, &map, &factions, &my_fac.name, &mut reactions);
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
    map: &Map,
    factions: &ReadStorage<Faction>,
    my_fac: &str,
    reactions: &mut Vec<(Entity, Reaction)>,
) {
    for other_entity in map.tile_content[idx as usize].iter() {
        if let Some(faction) = factions.get(*other_entity) {
            reactions.push((
                *other_entity,
                faction_reaction(my_fac, &faction.name, &RAWS.lock().unwrap()),
            ));
        }
    }
}
