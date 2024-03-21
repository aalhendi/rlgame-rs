use specs::{Entity, World, WorldExt};

use crate::{HungerClock, HungerState};

use super::EffectSpawner;

pub fn well_fed(ecs: &mut World, _damage: &EffectSpawner, target: Entity) {
    if let Some(hc) = ecs.write_storage::<HungerClock>().get_mut(target) {
        hc.state = HungerState::WellFed;
        hc.duration = 20;
    }
}
