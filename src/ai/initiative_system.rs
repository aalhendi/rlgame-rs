use specs::{Entities, Entity, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{Attributes, Initiative, MyTurn, Position, RunState};

pub struct InitiativeSystem;

impl<'a> System<'a> for InitiativeSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, Initiative>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, MyTurn>,
        Entities<'a>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadStorage<'a, Attributes>,
        WriteExpect<'a, RunState>,
        ReadExpect<'a, Entity>,
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
        ) = data;

        if *runstate != RunState::Ticking {
            return;
        }

        // Clear any remaining MyTurn we left by mistkae
        turns.clear();

        // Roll initiative
        for (entity, initiative, _pos) in (&entities, &mut initiatives, &positions).join() {
            initiative.current -= 1;

            // Not my turn
            if initiative.current >= 1 {
                continue;
            }
            // It's my turn!
            turns
                .insert(entity, MyTurn {})
                .expect("Unable to insert turn");

            // Re-roll (6 + 1d6 + Quickness Bonus)
            initiative.current = 6 + rng.roll_dice(1, 6);
            if let Some(attr) = attributes.get(entity) {
                initiative.current -= attr.quickness.bonus;
            }

            // TODO: More initiative granting boosts/penalties will go here later

            // If its the player, we want to go to an AwaitingInput state
            if entity == *player {
                *runstate = RunState::AwaitingInput;
            }
        }
    }
}
