use crate::{
    effects::{add_effect, EffectType, Targets},
    gamelog::Logger,
};

use super::{HungerClock, HungerState, MyTurn, RunState};
use specs::prelude::*;

pub struct HungerSystem;

impl<'a> System<'a> for HungerSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>, // The player
        ReadExpect<'a, RunState>,
        ReadStorage<'a, MyTurn>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity, _runstate, turns) = data;

        for (entity, clock, _myturn) in (&entities, &mut hunger_clock, &turns).join() {
            clock.duration -= 1;
            if clock.duration >= 1 {
                continue;
            }

            match clock.state {
                HungerState::WellFed => {
                    clock.state = HungerState::Normal;
                    clock.duration = 200;
                    if entity == *player_entity {
                        Logger::new().orange("You are no longer well fed").log();
                    }
                }
                HungerState::Normal => {
                    clock.state = HungerState::Hungry;
                    clock.duration = 200;
                    if entity == *player_entity {
                        Logger::new().orange("You are hungry").log();
                    }
                }
                HungerState::Hungry => {
                    clock.state = HungerState::Starving;
                    clock.duration = 200;
                    if entity == *player_entity {
                        Logger::new().red("You are starving!").log();
                    }
                }
                HungerState::Starving => {
                    // Inflict damage from hunger
                    if entity == *player_entity {
                        Logger::new()
                            .red("Your hunger pangs are getting painful! You suffer 1 hp damage.")
                            .log();
                    }
                    add_effect(
                        None,
                        EffectType::Damage { amount: 1 },
                        Targets::Single { target: entity },
                    );
                }
            }
        }
    }
}
