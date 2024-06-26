use rltk::RandomNumberGenerator;
use specs::{
    saveload::{MarkedBuilder, SimpleMarker},
    Builder, Entity, World, WorldExt,
};

use crate::{
    gamelog::{events::record_event, Logger},
    gamesystem::{mana_at_level, player_hp_at_level},
    spatial, Attributes, Confusion, DamageOverTime, Duration, EquipmentChanged, IsSerialized, Map,
    Name, Player, Pools, Skills, Slow, StatusEffect,
};

use super::{add_effect, targetting::entity_position, EffectSpawner, EffectType, Targets};

pub fn inflict_damage(ecs: &mut World, damage: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    let player_entity = ecs.fetch::<Entity>();

    if let Some(pool) = pools.get_mut(target) {
        if pool.god_mode {
            return;
        }

        // aoe spells dont hurt caster but can hurt friendlies
        // NOTE(aalhendi): This makes fireball too strong. Suggest an AffectsCaster{} tag that can be checked
        if damage.creator.is_some_and(|c| c == target) {
            return;
        }

        if let EffectType::Damage { amount } = damage.effect_type {
            pool.hit_points.current -= amount;
            add_effect(None, EffectType::Bloodstain, Targets::Single { target });
            add_effect(
                None,
                EffectType::Particle {
                    glyph: rltk::to_cp437('‼'),
                    fg: rltk::RGB::named(rltk::ORANGE),
                    bg: rltk::RGB::named(rltk::BLACK),
                    lifespan: 200.0,
                },
                Targets::Single { target },
            );

            if target == *player_entity {
                record_event("Damage Taken", amount);
            }
            if damage.creator == Some(*player_entity) {
                record_event("Damage Inflicted", amount);
            }

            if pool.hit_points.current < 1 {
                add_effect(
                    damage.creator,
                    EffectType::EntityDeath,
                    Targets::Single { target },
                );
            }
        }
    }
}

pub fn bloodstain(ecs: &mut World, tile_idx: i32) {
    let mut map = ecs.fetch_mut::<Map>();
    map.bloodstains.insert(tile_idx as usize);
}

pub fn death(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    let mut xp_gain = 0;
    let mut gold_gain = 0.0f32;

    let mut pools = ecs.write_storage::<Pools>();
    let mut attributes = ecs.write_storage::<Attributes>();

    if let Some(pos) = entity_position(ecs, target) {
        spatial::remove_entity(target, pos as usize);
    }

    if effect.creator.is_none() {
        return;
    }
    let source = effect.creator.unwrap();

    if ecs.read_storage::<Player>().get(source).is_some() {
        if let Some(stats) = pools.get(target) {
            xp_gain += stats.level * 100;
            gold_gain += stats.gold;
        }

        if xp_gain == 0 && gold_gain == 0.0 {
            return;
        }

        let player_stats = pools.get_mut(source).unwrap();
        let player_attributes = attributes.get_mut(source).unwrap();
        player_stats.xp += xp_gain;
        player_stats.gold += gold_gain;
        if player_stats.xp >= player_stats.level * 1000 {
            // We've gone up a level!
            player_stats.level += 1;
            Logger::new()
                .magenta(format!(
                    "Congratulations, you are now level {}",
                    player_stats.level
                ))
                .log();

            // Improve a random attribute
            let mut rng = ecs.fetch_mut::<RandomNumberGenerator>();
            let attr_to_boost = rng.roll_dice(1, 4);
            match attr_to_boost {
                1 => {
                    player_attributes.might.base += 1;
                    Logger::new().green("You feel stronger!").log();
                }

                2 => {
                    player_attributes.fitness.base += 1;
                    Logger::new().green("You feel healther!").log();
                }

                3 => {
                    player_attributes.quickness.base += 1;
                    Logger::new().green("You feel quicker!").log();
                }

                _ => {
                    player_attributes.intelligence.base += 1;
                    Logger::new().green("You feel smarter!").log();
                }
            }

            // Improve all skills
            let mut skills = ecs.write_storage::<Skills>();
            let player_skills = skills.get_mut(*ecs.fetch::<Entity>()).unwrap();
            for (_skill_name, skill_value) in player_skills.skills.iter_mut() {
                *skill_value += 1;
            }

            ecs.write_storage::<EquipmentChanged>()
                .insert(*ecs.fetch::<Entity>(), EquipmentChanged {})
                .expect("Insert failed");

            player_stats.hit_points.max = player_hp_at_level(
                player_attributes.fitness.base + player_attributes.fitness.modifiers,
                player_stats.level,
            );
            player_stats.hit_points.current = player_stats.hit_points.max;

            player_stats.mana.max = mana_at_level(
                player_attributes.intelligence.base + player_attributes.intelligence.modifiers,
                player_stats.level,
            );
            player_stats.mana.current = player_stats.mana.max;

            let player_pos = ecs.fetch::<rltk::Point>();
            let map = ecs.fetch::<Map>();
            for i in 0..10 {
                if player_pos.y - i > 1 {
                    add_effect(
                        None,
                        EffectType::Particle {
                            glyph: rltk::to_cp437('░'),
                            fg: rltk::RGB::named(rltk::GOLD),
                            bg: rltk::RGB::named(rltk::BLACK),
                            lifespan: 400.0,
                        },
                        Targets::Tile {
                            tile_idx: map.xy_idx(player_pos.x, player_pos.y - i) as i32,
                        },
                    );
                }
            }
        }
    }
}

pub fn heal_damage(ecs: &mut World, heal: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();

    let pool = pools.get_mut(target);
    if pool.is_none() {
        return;
    }
    let pool = pool.unwrap();

    if let EffectType::Healing { amount } = heal.effect_type {
        pool.hit_points.current = i32::min(pool.hit_points.max, pool.hit_points.current + amount);
        add_effect(
            None,
            EffectType::Particle {
                glyph: rltk::to_cp437('‼'),
                fg: rltk::RGB::named(rltk::GREEN),
                bg: rltk::RGB::named(rltk::BLACK),
                lifespan: 200.0,
            },
            Targets::Single { target },
        );
    }
}

pub fn add_confusion(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Confusion { turns } = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect { target })
            .with(Confusion {})
            .with(Duration { turns: *turns })
            .with(Name {
                name: "Confusion".to_string(),
            })
            .marked::<SimpleMarker<IsSerialized>>()
            .build();
    }
}

pub fn attribute_effect(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::AttributeEffect {
        bonus,
        name,
        duration,
    } = &effect.effect_type
    {
        ecs.create_entity()
            .with(StatusEffect { target })
            .with(bonus.clone())
            .with(Duration { turns: *duration })
            .with(Name { name: name.clone() })
            .marked::<SimpleMarker<IsSerialized>>()
            .build();
        ecs.write_storage::<EquipmentChanged>()
            .insert(target, EquipmentChanged {})
            .expect("Insert failed");
    }
}

pub fn restore_mana(ecs: &mut World, mana: &EffectSpawner, target: Entity) {
    let mut pools = ecs.write_storage::<Pools>();
    if let Some(pool) = pools.get_mut(target) {
        if let EffectType::Mana { amount } = mana.effect_type {
            pool.mana.current = i32::min(pool.mana.max, pool.mana.current + amount);
            add_effect(
                None,
                EffectType::Particle {
                    glyph: rltk::to_cp437('‼'),
                    fg: rltk::RGB::named(rltk::BLUE),
                    bg: rltk::RGB::named(rltk::BLACK),
                    lifespan: 200.0,
                },
                Targets::Single { target },
            );
        }
    }
}

pub fn slow(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::Slow { initiative_penalty } = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect { target })
            .with(Slow {
                initiative_penalty: *initiative_penalty,
            })
            .with(Duration { turns: 5 })
            .with(if *initiative_penalty > 0.0 {
                Name {
                    name: "Slowed".to_string(),
                }
            } else {
                Name {
                    name: "Hasted".to_string(),
                }
            })
            .marked::<SimpleMarker<IsSerialized>>()
            .build();
    }
}

pub fn damage_over_time(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    if let EffectType::DamageOverTime { damage } = &effect.effect_type {
        ecs.create_entity()
            .with(StatusEffect { target })
            .with(DamageOverTime { damage: *damage })
            .with(Duration { turns: 5 })
            .with(Name {
                name: "Damage Over Time".to_string(),
            })
            .marked::<SimpleMarker<IsSerialized>>()
            .build();
    }
}
