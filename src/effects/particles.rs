use specs::{Builder, World, WorldExt};

use crate::{
    particle_system::ParticleBuilder, Map, ParticleAnimation, ParticleLifetime, Position,
    Renderable,
};

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

pub fn projectile(ecs: &mut World, tile_idx: i32, effect: &EffectSpawner) {
    if let EffectType::ParticleProjectile {
        glyph,
        fg,
        bg,
        lifespan: _lifespan,
        speed,
        path,
    } = &effect.effect_type
    {
        let map = ecs.fetch::<Map>();
        let (x, y) = map.idx_xy(tile_idx as usize);
        std::mem::drop(map);
        ecs.create_entity()
            .with(Position { x, y })
            .with(Renderable {
                fg: *fg,
                bg: *bg,
                glyph: *glyph,
                render_order: 0,
            })
            .with(ParticleLifetime {
                lifetime_ms: path.len() as f32 * speed,
                animation: Some(ParticleAnimation {
                    step_time: *speed,
                    path: path.to_vec(),
                    current_step: 0,
                    timer: 0.0,
                }),
            })
            .build();
    }
}
