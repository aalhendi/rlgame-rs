use super::{
    gamelog::Gamelog, particle_system::ParticleBuilder, EntityMoved, EntryTrigger, Hidden,
    InflictsDamage, Map, Name, Position, SingleActivation, SufferDamage,
};
use specs::prelude::*;

pub struct TriggerSystem;

impl<'a> System<'a> for TriggerSystem {
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        WriteStorage<'a, Hidden>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, Gamelog>,
        ReadStorage<'a, InflictsDamage>,
        WriteExpect<'a, ParticleBuilder>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, SingleActivation>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entity_moved,
            position,
            entry_trigger,
            mut hidden,
            names,
            entities,
            mut log,
            inflicts_damage,
            mut particle_builder,
            mut suffer_damage,
            single_activation,
        ) = data;

        // Iterate the entities that moved and their final position
        let mut remove_entities: Vec<Entity> = Vec::new();
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            for tile_entity in map.tile_content[idx].iter() {
                // Is Triggerable
                if entity != *tile_entity && entry_trigger.get(*tile_entity).is_some() {
                    if let Some(name) = names.get(*tile_entity) {
                        log.entries.push(format!(
                            "{trigger_entity} triggers!",
                            trigger_entity = &name.name
                        ));
                    }

                    // Inflicts Damage
                    if let Some(damage) = inflicts_damage.get(*tile_entity) {
                        particle_builder.request(
                            *pos,
                            rltk::RGB::named(rltk::ORANGE),
                            rltk::RGB::named(rltk::BLACK),
                            rltk::to_cp437('‼'),
                            200.0,
                        );
                        SufferDamage::new_damage(&mut suffer_damage, entity, damage.damage);
                    }

                    // If it is single activation, it needs to be removed
                    if single_activation.get(*tile_entity).is_some() {
                        remove_entities.push(*tile_entity);
                    }

                    hidden.remove(*tile_entity); // The trap is no longer hidden
                }
            }
        }

        // Remove any single activation traps
        for trap in remove_entities.iter() {
            entities.delete(*trap).expect("Unable to delete trap");
        }

        // Remove all entity movement markers
        entity_moved.clear();
    }
}
