use std::collections::HashSet;

use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteStorage};

use crate::{
    effects::{add_effect, EffectType, Targets},
    Confusion, MyTurn, RunState, StatusEffect,
};

pub struct TurnStatusSystem;

impl<'a> System<'a> for TurnStatusSystem {
    type SystemData = (
        WriteStorage<'a, MyTurn>,
        ReadStorage<'a, Confusion>,
        Entities<'a>,
        ReadExpect<'a, RunState>,
        ReadStorage<'a, StatusEffect>,
    );

    fn run(&mut self, data: Self::SystemData) {
        // Iterates confused entities, decrements their turn counter. If still confused, removes MyTurn tag. If recovered, removes Confusion tag
        let (mut turns, confusion, entities, runstate, statuses) = data;

        if *runstate != RunState::Ticking {
            return;
        }

        // Collect a set of all entities whose turn it is
        let mut entity_turns = HashSet::new();
        for (entity, _turn) in (&entities, &turns).join() {
            entity_turns.insert(entity);
        }

        // Find status effects affecting entities whose turn it is
        let mut not_my_turn = Vec::new();
        for (effect_entity, status_effect) in (&entities, &statuses).join() {
            if entity_turns.contains(&status_effect.target) {
                // Skip turn for confusion
                if confusion.get(effect_entity).is_some() {
                    add_effect(
                        None,
                        EffectType::Particle {
                            glyph: rltk::to_cp437('?'),
                            fg: rltk::RGB::named(rltk::MAGENTA),
                            bg: rltk::RGB::named(rltk::BLACK),
                            lifespan: 200.0,
                        },
                        Targets::Single {
                            target: status_effect.target,
                        },
                    );
                    not_my_turn.push(status_effect.target);
                }
            }
        }

        for e in not_my_turn {
            turns.remove(e);
        }
    }
}
