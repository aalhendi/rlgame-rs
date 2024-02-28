use crate::{gamesystem::skill_bonus, Attributes, Pools, Skill, Skills};

use super::{
    gamelog::Gamelog, particle_system::ParticleBuilder, DefenseBonus, Equipped, HungerClock,
    HungerState, MeleePowerBonus, Name, Position, SufferDamage, WantsToMelee,
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;

pub struct MeleeCombatSystem;

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Attributes>,
        ReadStorage<'a, Skills>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, Gamelog>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Equipped>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
        ReadStorage<'a, Pools>,
        WriteExpect<'a, RandomNumberGenerator>,
    );
    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_melee,
            names,
            attributes,
            skills,
            mut inflict_damage,
            mut log,
            _melee_power_bonuses,
            _defense_bonuses,
            _equipped,
            mut particle_builder,
            positions,
            hunger_clock,
            pools,
            mut rng,
        ) = data;

        for (entity, wants_melee, name, attacker_attributes, attacker_skills, attacker_pools) in (
            &entities,
            &wants_melee,
            &names,
            &attributes,
            &skills,
            &pools,
        )
            .join()
        {
            let target_pools = pools.get(wants_melee.target).unwrap();
            let target_attributes = attributes.get(wants_melee.target).unwrap();
            let target_skills = skills.get(wants_melee.target).unwrap();
            // if attacker or target is dead, no need to calculate
            if attacker_pools.hit_points.current <= 0 || target_pools.hit_points.current <= 0 {
                continue;
            }

            let target_name = names.get(wants_melee.target).unwrap();

            let natural_roll = rng.roll_dice(1, 20);
            let attribute_hit_bonus = attacker_attributes.might.bonus;
            let skill_hit_bonus = skill_bonus(Skill::Melee, attacker_skills);
            let weapon_hit_bonus = 0; // TODO(aalhendi): Once weapons support this
            let mut status_hit_bonus = 0;
            if let Some(hc) = hunger_clock.get(entity) {
                // Well-Fed grants +1
                if hc.state == HungerState::WellFed {
                    status_hit_bonus += 1;
                }
            }
            let modified_hit_roll = natural_roll
                + attribute_hit_bonus
                + skill_hit_bonus
                + weapon_hit_bonus
                + status_hit_bonus;

            let base_armor_class = 10;
            let armor_quickness_bonus = target_attributes.quickness.bonus;
            let armor_skill_bonus = skill_bonus(Skill::Defense, target_skills);
            let armor_item_bonus = 0; // TODO(aalhendi): Once armor supports this
            let armor_class =
                base_armor_class + armor_quickness_bonus + armor_skill_bonus + armor_item_bonus;

            match natural_roll {
                // Natural 1 miss
                1 => {
                    log.entries.push(format!(
                        "{name} considers attacking {target_name}, but misjudges the timing.",
                        name = name.name,
                        target_name = target_name.name
                    ));
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(
                            *pos,
                            rltk::RGB::named(rltk::BLUE),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                    }
                }

                // Target hit! Until we support weapons, we're going with 1d4
                _ if natural_roll == 20 || modified_hit_roll > armor_class => {
                    let base_damage = rng.roll_dice(1, 4);
                    let attr_damage_bonus = attacker_attributes.might.bonus;
                    let skill_damage_bonus = skill_bonus(Skill::Melee, attacker_skills);
                    let weapon_damage_bonus = 0;

                    let damage = i32::max(
                        0,
                        base_damage
                            + attr_damage_bonus
                            + skill_hit_bonus
                            + skill_damage_bonus
                            + weapon_damage_bonus,
                    );
                    SufferDamage::new_damage(&mut inflict_damage, wants_melee.target, damage);
                    log.entries.push(format!(
                        "{name} hits {target_name}, for {damage} hp.",
                        name = &name.name,
                        target_name = &target_name.name,
                    ));
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(
                            *pos,
                            rltk::RGB::named(rltk::ORANGE),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                    }
                }

                // Miss
                _ => {
                    log.entries.push(format!(
                        "{name} attacks {target_name}, but can't connect.",
                        name = name.name,
                        target_name = target_name.name
                    ));
                    if let Some(pos) = positions.get(wants_melee.target) {
                        particle_builder.request(
                            *pos,
                            rltk::RGB::named(rltk::CYAN),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                    }
                }
            }
        }

        wants_melee.clear();
    }
}
