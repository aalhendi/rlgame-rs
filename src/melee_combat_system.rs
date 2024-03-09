use crate::{
    gamesystem::skill_bonus, Attributes, EquipmentSlot, NaturalAttackDefense, Pools, Skill, Skills,
    WeaponAttribute,
};

use super::{
    gamelog::Gamelog, particle_system::ParticleBuilder, Equipped, HungerClock, HungerState,
    MeleeWeapon, Name, Position, SufferDamage, WantsToMelee, Wearable,
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
        ReadStorage<'a, MeleeWeapon>,
        ReadStorage<'a, Wearable>,
        ReadStorage<'a, Equipped>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
        ReadStorage<'a, Pools>,
        ReadStorage<'a, NaturalAttackDefense>,
        WriteExpect<'a, RandomNumberGenerator>,
        ReadExpect<'a, Entity>,
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
            melee_weapons,
            wearables,
            equipped,
            mut particle_builder,
            positions,
            hunger_clock,
            pools,
            naturals,
            mut rng,
            player_entity,
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

            // Default weapon.
            // TODO(aalhendi): Refactor into MeleeWeapon::default()?
            let mut weapon_info = MeleeWeapon {
                attribute: WeaponAttribute::Might,
                damage_n_dice: 1,
                damage_die_type: 4,
                damage_bonus: 0,
                hit_bonus: 0,
            };

            // Check for natural attacks, pick one at random, mutate the weapon to match it
            if let Some(nat) = naturals.get(entity).filter(|nat| !nat.attacks.is_empty()) {
                let attack_index = if nat.attacks.len() == 1 {
                    0
                } else {
                    rng.roll_dice(1, nat.attacks.len() as i32) as usize - 1
                };
                weapon_info.hit_bonus = nat.attacks[attack_index].hit_bonus;
                weapon_info.damage_n_dice = nat.attacks[attack_index].damage_n_dice;
                weapon_info.damage_die_type = nat.attacks[attack_index].damage_die_type;
                weapon_info.damage_bonus = nat.attacks[attack_index].damage_bonus;
            }

            // If melee weapon, update its data
            for (equipment, melee) in (&equipped, &melee_weapons).join() {
                if equipment.owner == entity && equipment.slot == EquipmentSlot::Melee {
                    weapon_info = melee.clone();
                }
            }

            let natural_roll = rng.roll_dice(1, 20);
            let attribute_hit_bonus = match weapon_info.attribute {
                WeaponAttribute::Might => attacker_attributes.might.bonus,
                WeaponAttribute::Quickness => attacker_attributes.quickness.bonus,
            };
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

            // Calculate total armor item bonus
            // NOTE: Floats because D&D armor is per set. Here we can equip items seperately.
            let mut armor_item_bonus = 0_f32;
            for (equipment, armor) in (&equipped, &wearables).join() {
                if equipment.owner == wants_melee.target {
                    armor_item_bonus += armor.armor_class;
                }
            }

            let base_armor_class = match naturals.get(wants_melee.target) {
                Some(nat) => nat.armor_class.unwrap_or(10),
                None => 10,
            };
            let armor_quickness_bonus = target_attributes.quickness.bonus;
            let armor_skill_bonus = skill_bonus(Skill::Defense, target_skills);
            let armor_item_bonus = armor_item_bonus as i32;
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

                // Target hit!
                _ if natural_roll == 20 || modified_hit_roll > armor_class => {
                    let base_damage =
                        rng.roll_dice(weapon_info.damage_n_dice, weapon_info.damage_die_type);
                    let attr_damage_bonus = attacker_attributes.might.bonus;
                    let skill_damage_bonus = skill_bonus(Skill::Melee, attacker_skills);
                    let weapon_damage_bonus = weapon_info.damage_bonus;

                    let damage = i32::max(
                        0,
                        base_damage
                            + attr_damage_bonus
                            + skill_hit_bonus
                            + skill_damage_bonus
                            + weapon_damage_bonus,
                    );
                    SufferDamage::new_damage(
                        &mut inflict_damage,
                        wants_melee.target,
                        damage,
                        entity == *player_entity,
                    );
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
