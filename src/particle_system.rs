use super::{ParticleLifetime, Position, Renderable, Rltk};
use specs::prelude::*;

pub fn update_particles(ecs: &mut World, ctx: &Rltk) {
    let mut dead_particles = Vec::new();
    {
        // Age out particles
        let mut particles = ecs.write_storage::<ParticleLifetime>();
        let entities = ecs.entities();
        for (entity, particle) in (&entities, &mut particles).join() {
            if let Some(animation) = &mut particle.animation {
                animation.timer += ctx.frame_time_ms;
                if animation.timer > animation.step_time
                    && animation.current_step < animation.path.len() - 2
                {
                    animation.current_step += 1;

                    if let Some(pos) = ecs.write_storage::<Position>().get_mut(entity) {
                        pos.x = animation.path[animation.current_step].x;
                        pos.y = animation.path[animation.current_step].y;
                    }
                }
            }

            particle.lifetime_ms -= ctx.frame_time_ms;
            if particle.lifetime_ms < 0.0 {
                dead_particles.push(entity);
            }
        }
    }
    for dead in dead_particles.iter() {
        ecs.delete_entity(*dead).expect("Particle will not die");
    }
}

struct ParticleRequest {
    pos: Position,
    fg: rltk::RGB,
    bg: rltk::RGB,
    glyph: rltk::FontCharType,
    lifetime: f32,
}

#[derive(Default)]
pub struct ParticleBuilder {
    requests: Vec<ParticleRequest>,
}

impl ParticleBuilder {
    pub fn new() -> ParticleBuilder {
        ParticleBuilder {
            requests: Vec::new(),
        }
    }

    pub fn request(
        &mut self,
        pos: Position,
        fg: rltk::RGB,
        bg: rltk::RGB,
        glyph: rltk::FontCharType,
        lifetime: f32,
    ) {
        self.requests.push(ParticleRequest {
            pos,
            fg,
            bg,
            glyph,
            lifetime,
        });
    }
}

pub struct ParticleSpawnSystem;

impl<'a> System<'a> for ParticleSpawnSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Renderable>,
        WriteStorage<'a, ParticleLifetime>,
        WriteExpect<'a, ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut positions, mut renderables, mut particles, mut particle_builder) = data;
        for new_particle in particle_builder.requests.iter() {
            let particle = entities.create();
            positions
                .insert(particle, new_particle.pos)
                .expect("Unable to insert position");

            renderables
                .insert(
                    particle,
                    Renderable {
                        fg: new_particle.fg,
                        bg: new_particle.bg,
                        glyph: new_particle.glyph,
                        render_order: 0,
                    },
                )
                .expect("Unable to insert renderable");

            particles
                .insert(
                    particle,
                    ParticleLifetime {
                        lifetime_ms: new_particle.lifetime,
                        animation: None,
                    },
                )
                .expect("Unable to insert lifetime");
        }

        particle_builder.requests.clear();
    }
}
