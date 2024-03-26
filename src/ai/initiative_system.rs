use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    effects::{add_effect, EffectType, Targets},
    Attributes, DamageOverTime, Duration, EquipmentChanged, Initiative, MyTurn, Pools, Position,
    RunState, StatusEffect,
};

pub struct InitiativeSystem;

impl<'a> System<'a> for InitiativeSystem {
    type SystemData = (
        WriteStorage<'a, Initiative>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, MyTurn>,
        Entities<'a>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, RunState>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, rltk::Point>,
        ReadStorage<'a, Pools>,
        ReadStorage<'a, StatusEffect>,
        WriteStorage<'a, EquipmentChanged>,
        WriteStorage<'a, Duration>,
        ReadStorage<'a, DamageOverTime>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut initiatives,
            positions,
            mut turns,
            entities,
            mut rng,
            attributes,
            mut runstate,
            player,
            player_pos,
            pools,
            statuses,
            mut equipment_dirty,
            mut durations,
            dots,
        ) = data;

        if *runstate != RunState::Ticking {
            return;
        }

        // Clear any remaining MyTurn we left by mistkae
        turns.clear();

        // Roll initiative
        for (entity, initiative, pos) in (&entities, &mut initiatives, &positions).join() {
            initiative.current -= 1;

            // Not my turn
            if initiative.current >= 1 {
                continue;
            }

            let mut my_turn = true;

            // Re-roll (6 + 1d6 + Quickness Bonus)
            initiative.current = 6 + rng.roll_dice(1, 6);
            if let Some(attr) = attributes.get(entity) {
                initiative.current -= attr.quickness.bonus;
            }

            // Apply pool penalty
            if let Some(pools) = pools.get(entity) {
                initiative.current += f32::floor(pools.total_initiative_penalty) as i32;
            }
            // TODO: More initiative granting boosts/penalties will go here later

            // If its the player, we want to go to an AwaitingInput state
            if entity == *player {
                *runstate = RunState::AwaitingInput;
            } else {
                let e_pos = rltk::Point::new(pos.x, pos.y);
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, e_pos);
                if distance > 20.0 {
                    my_turn = false;
                }
            }

            // It's my turn!
            if my_turn {
                turns
                    .insert(entity, MyTurn {})
                    .expect("Unable to insert turn");
            }
        }

        // Handle durations
        if *runstate == RunState::AwaitingInput {
            for (effect_entity, duration, status) in (&entities, &mut durations, &statuses).join() {
                // Status effects might out-live their target, if entity is dead we might crash since entity will be invalid
                if entities.is_alive(status.target) {
                    duration.turns -= 1;
                    // DOT could be its own system but doesn't seem too important.
                    if let Some(dot) = dots.get(effect_entity) {
                        add_effect(
                            None,
                            EffectType::Damage { amount: dot.damage },
                            Targets::Single {
                                target: status.target,
                            },
                        );
                    }
                    if duration.turns < 1 {
                        equipment_dirty
                            .insert(status.target, EquipmentChanged {})
                            .expect("Unable to insert");
                        entities.delete(effect_entity).expect("Unable to delete");
                    }
                }
            }
        }
    }
}
