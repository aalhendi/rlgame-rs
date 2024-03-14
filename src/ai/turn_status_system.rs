use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{particle_system::ParticleBuilder, Confusion, MyTurn, Position, RunState};

pub struct TurnStatusSystem;

impl<'a> System<'a> for TurnStatusSystem {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        WriteStorage<'a, Confusion>,
        Entities<'a>,
        ReadExpect<'a, RunState>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        // Iterates confused entities, decrements their turn counter. If still confused, removes MyTurn tag. If recovered, removes Confusion tag
        let (mut turns, mut confusion, entities, runstate, mut particle_builder, positions) = data;

        if *runstate != RunState::Ticking {
            return;
        }

        let mut not_my_turn = Vec::new();
        let mut not_confused = Vec::new();
        for (entity, _turn, confused) in (&entities, &mut turns, &mut confusion).join() {
            confused.turns -= 1;
            if confused.turns < 1 {
                not_confused.push(entity);
            } else {
                let pos = positions.get(entity).unwrap();
                particle_builder.request(
                    *pos,
                    rltk::RGB::named(rltk::MAGENTA),
                    rltk::RGB::named(rltk::BLACK),
                    rltk::to_cp437('?'),
                    200.0,
                );
                not_my_turn.push(entity);
            }
        }

        for e in not_my_turn {
            turns.remove(e);
        }

        for e in not_confused {
            confusion.remove(e);
        }
    }
}
