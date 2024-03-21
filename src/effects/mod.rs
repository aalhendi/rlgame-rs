use std::collections::VecDeque;
use std::sync::Mutex;

use specs::{Entity, World};

use crate::spatial;

use self::targetting::entity_position;

mod damage;
mod hunger;
mod movement;
mod particles;
pub mod targetting;
mod triggers;

lazy_static! {
    pub static ref EFFECT_QUEUE: Mutex<VecDeque<EffectSpawner>> = Mutex::new(VecDeque::new());
}

pub enum EffectType {
    Damage {
        amount: i32,
    },
    Bloodstain,
    Particle {
        glyph: rltk::FontCharType,
        fg: rltk::RGB,
        bg: rltk::RGB,
        lifespan: f32,
    },
    // ParticleProjectile {
    //     glyph: rltk::FontCharType,
    //     fg: rltk::RGB,
    //     bg: rltk::RGB,
    //     lifespan: f32,
    //     speed: f32,
    //     path: Vec<rltk::Point>,
    // },
    EntityDeath,
    ItemUse {
        item: Entity,
    },
    // SpellUse {
    //     spell: Entity,
    // },
    WellFed,
    Healing {
        amount: i32,
    },
    // Mana {
    //     amount: i32,
    // },
    Confusion {
        turns: i32,
    },
    TriggerFire {
        trigger: Entity,
    },
    TeleportTo {
        x: i32,
        y: i32,
        depth: i32,
        player_only: bool,
    },
    // AttributeEffect {
    //     bonus: AttributeBonus,
    //     name: String,
    //     duration: i32,
    // },
    // Slow {
    //     initiative_penalty: f32,
    // },
    // DamageOverTime {
    //     damage: i32,
    // },
}

#[derive(Clone)]
pub enum Targets {
    Single { target: Entity },
    _TargetList { targets: Vec<Entity> },
    Tile { tile_idx: i32 },
    Tiles { tiles: Vec<i32> },
}

pub struct EffectSpawner {
    pub creator: Option<Entity>,
    pub effect_type: EffectType,
    pub targets: Targets,
}

pub fn add_effect(creator: Option<Entity>, effect_type: EffectType, targets: Targets) {
    EFFECT_QUEUE.lock().unwrap().push_back(EffectSpawner {
        creator,
        effect_type,
        targets,
    });
}

pub fn run_effects_queue(ecs: &mut World) {
    loop {
        let lock = EFFECT_QUEUE.lock().unwrap().pop_front();
        if let Some(effect) = lock {
            target_applicator(ecs, &effect);
        } else {
            break;
        }
    }
}

fn target_applicator(ecs: &mut World, effect: &EffectSpawner) {
    if let EffectType::ItemUse { item } = effect.effect_type {
        triggers::item_trigger(effect.creator, item, &effect.targets, ecs);
        return;
    }

    if let EffectType::TriggerFire { trigger } = effect.effect_type {
        triggers::trigger(effect.creator, trigger, &effect.targets, ecs);
        return;
    }

    match &effect.targets {
        Targets::Tile { tile_idx } => affect_tile(ecs, effect, *tile_idx),
        Targets::Tiles { tiles } => tiles
            .iter()
            .for_each(|tile_idx| affect_tile(ecs, effect, *tile_idx)),
        Targets::Single { target } => affect_entity(ecs, effect, *target),
        Targets::_TargetList { targets } => targets
            .iter()
            .for_each(|entity| affect_entity(ecs, effect, *entity)),
    }
}

fn tile_effect_hits_entities(effect: &EffectType) -> bool {
    matches!(
        effect,
        EffectType::Damage { .. }
            | EffectType::WellFed
            | EffectType::Healing { .. }
            // | EffectType::Mana { .. }
            | EffectType::Confusion { .. }
            | EffectType::TeleportTo { .. } // | EffectType::AttributeEffect { .. }
                                            // | EffectType::Slow { .. }
                                            // | EffectType::DamageOverTime { .. }
    )
}

fn affect_tile(ecs: &mut World, effect: &EffectSpawner, tile_idx: i32) {
    if tile_effect_hits_entities(&effect.effect_type) {
        spatial::for_each_tile_content(tile_idx as usize, |entity| {
            affect_entity(ecs, effect, entity)
        });
    }

    match effect.effect_type {
        EffectType::Bloodstain => damage::bloodstain(ecs, tile_idx),
        EffectType::Particle { .. } => particles::particle_to_tile(ecs, tile_idx, effect),
        _ => (),
    }
}

fn affect_entity(ecs: &mut World, effect: &EffectSpawner, target: Entity) {
    match &effect.effect_type {
        EffectType::Damage { .. } => damage::inflict_damage(ecs, effect, target),
        EffectType::Bloodstain { .. } => {
            if let Some(pos) = entity_position(ecs, target) {
                damage::bloodstain(ecs, pos)
            }
        }
        EffectType::Particle { .. } => {
            if let Some(pos) = entity_position(ecs, target) {
                particles::particle_to_tile(ecs, pos, effect)
            }
        }
        EffectType::EntityDeath { .. } => damage::death(ecs, effect, target),
        EffectType::WellFed => hunger::well_fed(ecs, effect, target),
        EffectType::Healing { .. } => damage::heal_damage(ecs, effect, target),
        // EffectType::Mana { .. } => damage::restore_mana(ecs, effect, target),
        EffectType::Confusion { .. } => damage::add_confusion(ecs, effect, target),
        EffectType::TeleportTo { .. } => movement::apply_teleport(ecs, effect, target),
        // EffectType::AttributeEffect { .. } => damage::attribute_effect(ecs, effect, target),
        // EffectType::Slow { .. } => damage::slow(ecs, effect, target),
        // EffectType::DamageOverTime { .. } => damage::damage_over_time(ecs, effect, target),
        // EffectType::ParticleProjectile { .. }
        EffectType::ItemUse { .. }
        // | EffectType::SpellUse { .. }
        | EffectType::TriggerFire { .. } => (),
    }
}
