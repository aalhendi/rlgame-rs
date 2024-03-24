use specs::{Entity, World, WorldExt};

use crate::{
    gamelog::Gamelog, AttributeBonus, Confusion, Consumable, Duration, Hidden, InflictsDamage,
    MagicMapper, Map, Name, ProvidesFood, ProvidesHealing, ProvidesIdentification,
    ProvidesRemoveCurse, RunState, SingleActivation, SpawnParticleBurst, SpawnParticleLine,
    TeleportTo, TownPortal,
};

use super::{
    add_effect,
    targetting::{entity_position, find_item_position},
    EffectType, Targets,
};

pub fn item_trigger(creator: Option<Entity>, item: Entity, targets: &Targets, ecs: &mut World) {
    // Check charges
    if let Some(c) = ecs.write_storage::<Consumable>().get_mut(item) {
        match c.charges.cmp(&1) {
            std::cmp::Ordering::Less => {
                // Cancel
                let mut gamelog = ecs.fetch_mut::<Gamelog>();
                gamelog.entries.push(format!(
                    "{} is out of charges!",
                    ecs.read_storage::<Name>().get(item).unwrap().name
                ));
                return;
            }
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => c.charges -= 1,
        }
    }

    // Use the item via the generic system
    let did_something = event_trigger(creator, item, targets, ecs);

    // If it was a consumable, then it gets deleted
    if did_something && ecs.read_storage::<Consumable>().get(item).is_some() {
        ecs.entities().delete(item).expect("Delete Failed");
    }
}

pub fn trigger(creator: Option<Entity>, trigger: Entity, targets: &Targets, ecs: &mut World) {
    // The triggering item is no longer hidden
    ecs.write_storage::<Hidden>().remove(trigger);

    // Use the item via the generic system
    let did_something = event_trigger(creator, trigger, targets, ecs);

    // If it was a single activation, then it gets deleted
    if did_something
        && ecs
            .read_storage::<SingleActivation>()
            .get(trigger)
            .is_some()
    {
        ecs.entities().delete(trigger).expect("Delete Failed");
    }
}

fn event_trigger(
    creator: Option<Entity>,
    entity: Entity,
    targets: &Targets,
    ecs: &mut World,
) -> bool {
    let mut did_something = false;
    let mut gamelog = ecs.fetch_mut::<Gamelog>();

    // Simple particle spawn
    if let Some(part) = ecs.read_storage::<SpawnParticleBurst>().get(entity) {
        add_effect(
            creator,
            EffectType::Particle {
                glyph: part.glyph,
                fg: part.color,
                bg: rltk::RGB::named(rltk::BLACK),
                lifespan: part.lifetime_ms,
            },
            targets.clone(),
        );
    }

    // Line particle spawn
    if let Some(part) = ecs.read_storage::<SpawnParticleLine>().get(entity) {
        if let Some(start_pos) = find_item_position(ecs, entity) {
            match targets {
                Targets::Tile { tile_idx } => spawn_line_particles(ecs, start_pos, *tile_idx, part),
                Targets::Tiles { tiles } => tiles
                    .iter()
                    .for_each(|tile_idx| spawn_line_particles(ecs, start_pos, *tile_idx, part)),
                Targets::Single { target } => {
                    if let Some(end_pos) = entity_position(ecs, *target) {
                        spawn_line_particles(ecs, start_pos, end_pos, part);
                    }
                }
                Targets::_TargetList { targets } => {
                    targets.iter().for_each(|target| {
                        if let Some(end_pos) = entity_position(ecs, *target) {
                            spawn_line_particles(ecs, start_pos, end_pos, part);
                        }
                    });
                }
            }
        }
    }

    // Providing food
    if ecs.read_storage::<ProvidesFood>().get(entity).is_some() {
        add_effect(creator, EffectType::WellFed, targets.clone());
        let names = ecs.read_storage::<Name>();
        gamelog
            .entries
            .push(format!("You eat the {}.", names.get(entity).unwrap().name));
        did_something = true;
    }

    // Magic mapper
    if ecs.read_storage::<MagicMapper>().get(entity).is_some() {
        let mut runstate = ecs.fetch_mut::<RunState>();
        gamelog
            .entries
            .push("The map is revealed to you!".to_string());
        *runstate = RunState::MagicMapReveal { row: 0 };
        did_something = true;
    }

    // Remove Curse
    if ecs
        .read_storage::<ProvidesRemoveCurse>()
        .get(entity)
        .is_some()
    {
        let mut runstate = ecs.fetch_mut::<RunState>();
        *runstate = RunState::ShowRemoveCurse;
        did_something = true;
    }

    // Identify Item
    if ecs
        .read_storage::<ProvidesIdentification>()
        .get(entity)
        .is_some()
    {
        let mut runstate = ecs.fetch_mut::<RunState>();
        *runstate = RunState::ShowIdentify;
        did_something = true;
    }

    // Town Portal
    if ecs.read_storage::<TownPortal>().get(entity).is_some() {
        let map = ecs.fetch::<Map>();
        if map.depth == 1 {
            gamelog
                .entries
                .push("You are already in town, so the scroll does nothing.".to_string());
        } else {
            gamelog
                .entries
                .push("You are telported back to town!".to_string());
            let mut runstate = ecs.fetch_mut::<RunState>();
            *runstate = RunState::TownPortal;
            did_something = true;
        }
    }

    // Healing
    if let Some(heal) = ecs.read_storage::<ProvidesHealing>().get(entity) {
        add_effect(
            creator,
            EffectType::Healing {
                amount: heal.heal_amount,
            },
            targets.clone(),
        );
        did_something = true;
    }

    // Damage
    if let Some(damage) = ecs.read_storage::<InflictsDamage>().get(entity) {
        add_effect(
            creator,
            EffectType::Damage {
                amount: damage.damage,
            },
            targets.clone(),
        );
        did_something = true;
    }

    // Confusion
    if let (Some(_confusion), Some(duration)) = (
        ecs.read_storage::<Confusion>().get(entity),
        ecs.read_storage::<Duration>().get(entity),
    ) {
        add_effect(
            creator,
            EffectType::Confusion {
                turns: duration.turns,
            },
            targets.clone(),
        );
        did_something = true;
    }

    // Teleport
    if let Some(teleport) = ecs.read_storage::<TeleportTo>().get(entity) {
        add_effect(
            creator,
            EffectType::TeleportTo {
                x: teleport.x,
                y: teleport.y,
                depth: teleport.depth,
                player_only: teleport.player_only,
            },
            targets.clone(),
        );
        did_something = true;
    }

    // Attribute Modifiers
    if let Some(attr) = ecs.read_storage::<AttributeBonus>().get(entity) {
        add_effect(
            creator,
            EffectType::AttributeEffect {
                bonus: attr.clone(),
                duration: 10,
                name: ecs.read_storage::<Name>().get(entity).unwrap().name.clone(),
            },
            targets.clone(),
        );
        did_something = true;
    }

    did_something
}

fn spawn_line_particles(ecs: &World, start: i32, end: i32, part: &SpawnParticleLine) {
    let map = ecs.fetch::<Map>();
    let start_pt = rltk::Point::new(start % map.width, end / map.width);
    let end_pt = rltk::Point::new(end % map.width, end / map.width);
    let line = rltk::line2d(rltk::LineAlg::Bresenham, start_pt, end_pt);
    for pt in line.iter() {
        add_effect(
            None,
            EffectType::Particle {
                glyph: part.glyph,
                fg: part.color,
                bg: rltk::RGB::named(rltk::BLACK),
                lifespan: part.lifetime_ms,
            },
            Targets::Tile {
                tile_idx: map.xy_idx(pt.x, pt.y) as i32,
            },
        );
    }
}
