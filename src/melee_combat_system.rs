use super::{
    gamelog::Gamelog, particle_system::ParticleBuilder, CombatStats, DefenseBonus, Equipped,
    HungerClock, HungerState, MeleePowerBonus, Name, Position, SufferDamage, WantsToMelee,
};
use specs::prelude::*;

pub struct MeleeCombatSystem;

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, Gamelog>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Equipped>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_melee,
            names,
            combat_stats,
            mut inflict_damage,
            mut log,
            melee_power_bonuses,
            defense_bonuses,
            equipped,
            mut particle_builder,
            positions,
            hunger_clock,
        ) = data;

        for (entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            if stats.hp > 0 {
                let mut offensive_bonus = 0;
                for (_item_entity, power_bonus, equipped_by) in
                    (&entities, &melee_power_bonuses, &equipped).join()
                {
                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.amount;
                    }
                }

                let target_stats = combat_stats.get(wants_melee.target).unwrap(); // TODO: Error handling
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();
                    let mut defensive_bonus = 0;
                    for (_item_entity, defense_bonus, equipped_by) in
                        (&entities, &defense_bonuses, &equipped).join()
                    {
                        if equipped_by.owner == wants_melee.target {
                            defensive_bonus += defense_bonus.amount;
                        }
                    }

                    if let Some(hc) = hunger_clock.get(entity) {
                        if hc.state == HungerState::WellFed {
                            offensive_bonus += 1;
                        }
                    }

                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(
                            *pos,
                            rltk::RGB::named(rltk::ORANGE),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                    }

                    let damage = i32::max(
                        0,
                        (stats.power + offensive_bonus) - (target_stats.defense + defensive_bonus),
                    );

                    if damage == 0 {
                        log.entries.push(format!(
                            "{name} is unable to hurt {target_name}",
                            name = &name.name,
                            target_name = &target_name.name
                        ));
                    } else {
                        log.entries.push(format!(
                            "{name} hits {target_name} for {damage} hp.",
                            name = &name.name,
                            target_name = &target_name.name
                        ));
                        SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    }
                }
            }
        }

        wants_melee.clear();
    }
}
