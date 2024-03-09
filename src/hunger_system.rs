use super::{gamelog::Gamelog, HungerClock, HungerState, RunState, SufferDamage};
use specs::prelude::*;

pub struct HungerSystem;

impl<'a> System<'a> for HungerSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>, // The player
        ReadExpect<'a, RunState>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, Gamelog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity, runstate, mut inflict_damage, mut log) =
            data;

        for (entity, clock) in (&entities, &mut hunger_clock).join() {
            let mut proceed = false;

            match *runstate {
                RunState::PlayerTurn => {
                    if entity == *player_entity {
                        proceed = true;
                    }
                }
                RunState::MonsterTurn => {
                    if entity != *player_entity {
                        proceed = true;
                    }
                }
                _ => proceed = false,
            }

            if !proceed {
                return;
            }

            clock.duration -= 1;
            if clock.duration >= 1 {
                return;
            }

            match clock.state {
                HungerState::WellFed => {
                    clock.state = HungerState::Normal;
                    clock.duration = 200;
                    if entity == *player_entity {
                        log.entries.push("You are no longer well fed.".to_string());
                    }
                }
                HungerState::Normal => {
                    clock.state = HungerState::Hungry;
                    clock.duration = 200;
                    if entity == *player_entity {
                        log.entries.push("You are hungry.".to_string());
                    }
                }
                HungerState::Hungry => {
                    clock.state = HungerState::Starving;
                    clock.duration = 200;
                    if entity == *player_entity {
                        log.entries.push("You are starving!".to_string());
                    }
                }
                HungerState::Starving => {
                    // Inflict damage from hunger
                    if entity == *player_entity {
                        log.entries.push(
                            "Your hunger pangs are getting painful! You suffer 1 hp damage."
                                .to_string(),
                        );
                    }
                    SufferDamage::new_damage(&mut inflict_damage, entity, 1, false);
                }
            }
        }
    }
}
