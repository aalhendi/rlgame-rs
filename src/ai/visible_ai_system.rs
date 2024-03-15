use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteStorage};

use crate::{
    raws::{faction_structs::Reaction, rawsmaster::faction_reaction, RAWS},
    Chasing, Faction, Map, MyTurn, Position, Viewshed, WantsToApproach, WantsToFlee,
};

pub struct VisibleAI;

impl<'a> System<'a> for VisibleAI {
    type SystemData = (
        ReadStorage<'a, MyTurn>,
        ReadStorage<'a, Faction>,
        ReadStorage<'a, Position>,
        ReadExpect<'a, Map>,
        WriteStorage<'a, WantsToApproach>,
        WriteStorage<'a, WantsToFlee>,
        Entities<'a>,
        ReadExpect<'a, Entity>,
        ReadStorage<'a, Viewshed>,
        WriteStorage<'a, Chasing>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            turns,
            factions,
            positions,
            map,
            mut want_approach,
            mut want_flee,
            entities,
            player,
            viewsheds,
            mut chasing,
        ) = data;

        for (entity, _turn, my_faction, pos, viewshed) in
            (&entities, &turns, &factions, &positions, &viewsheds).join()
        {
            if entity == *player {
                continue;
            }

            let my_idx = map.xy_idx(pos.x, pos.y);
            let mut reactions = Vec::new();
            let mut flee = Vec::new();
            for visible_tile in viewshed.visible_tiles.iter() {
                let idx = map.xy_idx(visible_tile.x, visible_tile.y);
                if my_idx != idx {
                    evaluate(idx, &map, &factions, &my_faction.name, &mut reactions);
                }
            }

            let mut done = false;
            for (tgt_idx, reaction, tgt_entity) in reactions {
                match reaction {
                    Reaction::Attack => {
                        want_approach
                            .insert(
                                entity,
                                WantsToApproach {
                                    idx: tgt_idx as i32,
                                },
                            )
                            .expect("Unable to insert");
                        chasing
                            .insert(entity, Chasing { target: tgt_entity })
                            .expect("Unable to insert");
                        done = true;
                    }
                    Reaction::Flee => {
                        flee.push(tgt_idx);
                    }
                    _ => {}
                }
            }

            if !done && !flee.is_empty() {
                want_flee
                    .insert(entity, WantsToFlee { indices: flee })
                    .expect("Unable to insert");
            }
        }
    }
}

fn evaluate(
    idx: usize,
    map: &Map,
    factions: &ReadStorage<Faction>,
    my_fac: &str,
    reactions: &mut Vec<(usize, Reaction, Entity)>,
) {
    for other_entity in map.tile_content[idx].iter() {
        if let Some(faction) = factions.get(*other_entity) {
            reactions.push((
                idx,
                faction_reaction(my_fac, &faction.name, &RAWS.lock().unwrap()),
                *other_entity,
            ));
        }
    }
}
