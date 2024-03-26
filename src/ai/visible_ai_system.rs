use rltk::{DistanceAlg, Point};
use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    raws::{faction_structs::Reaction, rawsmaster::faction_reaction, RAWS},
    spatial, Chasing, Faction, Map, MyTurn, Name, Position, SpecialAbilities, SpellTemplate,
    Viewshed, WantsToApproach, WantsToCastSpell, WantsToFlee,
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
        ReadStorage<'a, SpecialAbilities>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        WriteStorage<'a, WantsToCastSpell>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, SpellTemplate>,
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
            abilities,
            mut rng,
            mut casting,
            names,
            spells,
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
                    evaluate(idx, &factions, &my_faction.name, &mut reactions);
                }
            }

            let mut done = false;
            for (tgt_idx, reaction, tgt_entity) in reactions {
                match reaction {
                    // TODO(aalhendi): Refactor!
                    Reaction::Attack => {
                        if let Some(abilities) = abilities.get(entity) {
                            let (end_x, end_y) = map.idx_xy(tgt_idx);
                            let end_point = Point::new(end_x, end_y);
                            let range = DistanceAlg::Pythagoras
                                .distance2d(Point::new(pos.x, pos.y), end_point);
                            for ability in abilities.abilities.iter() {
                                if range >= ability.min_range
                                    && range <= ability.range
                                    && rng.roll_dice(1, 100) >= (ability.chance * 100.0) as i32
                                {
                                    use crate::raws::rawsmaster::find_spell_entity_by_name;
                                    casting
                                        .insert(
                                            entity,
                                            WantsToCastSpell {
                                                spell: find_spell_entity_by_name(
                                                    &ability.spell,
                                                    &names,
                                                    &spells,
                                                    &entities,
                                                )
                                                .unwrap(),
                                                target: Some(end_point),
                                            },
                                        )
                                        .expect("Unable to insert");
                                    done = true;
                                }
                            }
                        }

                        if !done {
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
    factions: &ReadStorage<Faction>,
    my_fac: &str,
    reactions: &mut Vec<(usize, Reaction, Entity)>,
) {
    spatial::for_each_tile_content(idx, |other_entity| {
        if let Some(faction) = factions.get(other_entity) {
            reactions.push((
                idx,
                faction_reaction(my_fac, &faction.name, &RAWS.lock().unwrap()),
                other_entity,
            ));
        }
    });
}
