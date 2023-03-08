use super::{ParticleLifetime, Position, Renderable, Rltk};
use specs::prelude::*;

pub fn cull_dead_particles(ecs: &mut World, ctx: &Rltk) {
    let mut dead_particles: Vec<Entity> = Vec::new();
    {
        let mut particles = ecs.write_storage::<ParticleLifetime>();
        let entities = ecs.entities();
        for (entity, mut particle) in (&entities, &mut particles).join() {
            particle.lifetime_ms -= ctx.frame_time_ms;
            if particle.lifetime_ms < 0. {
                dead_particles.push(entity);
            }
        }
    }
    for dead_particle in dead_particles.iter() {
        ecs.delete_entity(*dead_particle)
            .expect("Unable to delete particle")
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
                .insert(particle, new_particle.pos.clone())
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
                    },
                )
                .expect("Unable to insert lifetime");
        }

        particle_builder.requests.clear();
    }
}
