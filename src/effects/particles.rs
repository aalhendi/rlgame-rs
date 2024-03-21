use specs::World;

use crate::{particle_system::ParticleBuilder, Map, Position};

use super::{EffectSpawner, EffectType};

pub fn particle_to_tile(ecs: &mut World, tile_idx: i32, effect: &EffectSpawner) {
    if let EffectType::Particle {
        glyph,
        fg,
        bg,
        lifespan,
    } = effect.effect_type
    {
        let map = ecs.fetch::<Map>();
        let mut particle_builder = ecs.fetch_mut::<ParticleBuilder>();
        let (x, y) = map.idx_xy(tile_idx as usize);
        let pos = Position { x, y };
        particle_builder.request(pos, fg, bg, glyph, lifespan);
    }
}
