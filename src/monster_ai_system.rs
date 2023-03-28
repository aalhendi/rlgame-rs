use super::{
    particle_system::ParticleBuilder, Confusion, EntityMoved, Map, Monster, Position, RunState,
    Viewshed, WantsToMelee,
};
use rltk::Point;
use specs::prelude::*;

pub struct MonsterAI;

type MonsterAIData<'a> = (
    WriteExpect<'a, Map>,
    ReadExpect<'a, Point>,
    ReadExpect<'a, Entity>,
    Entities<'a>,
    WriteStorage<'a, Viewshed>,
    ReadStorage<'a, Monster>,
    WriteStorage<'a, Position>,
    WriteStorage<'a, WantsToMelee>,
    ReadExpect<'a, RunState>,
    WriteStorage<'a, Confusion>,
    WriteExpect<'a, ParticleBuilder>,
    WriteStorage<'a, EntityMoved>,
);
impl<'a> System<'a> for MonsterAI {
    type SystemData = MonsterAIData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            entities,
            mut viewshed,
            monster,
            mut position,
            mut wants_to_melee,
            runstate,
            mut confused,
            mut particle_builder,
            mut entity_moved,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }

        for (entity, mut viewshed, _monster, mut pos) in
            (&entities, &mut viewshed, &monster, &mut position).join()
        {
            let mut can_act = true;

            if let Some(am_confused) = confused.get_mut(entity) {
                am_confused.turns -= 1;
                if am_confused.turns < 1 {
                    confused.remove(entity);
                }
                can_act = false;
                particle_builder.request(
                    *pos,
                    rltk::RGB::named(rltk::MAGENTA),
                    rltk::RGB::named(rltk::BLACK),
                    rltk::to_cp437('?'),
                    200.0,
                );
            }

            if can_act {
                let distance =
                    rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance < 1.5 {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *player_entity,
                            },
                        )
                        .expect("Unable to insert attack");
                } else if viewshed.visible_tiles.contains(&*player_pos) {
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y) as i32,
                        map.xy_idx(player_pos.x, player_pos.y) as i32,
                        &*map,
                    );
                    if path.success && path.steps.len() > 1 {
                        // Clear old pos
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;
                        // Calc new pos
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        // Set new pos
                        entity_moved
                            .insert(entity, EntityMoved {})
                            .expect("Unable to insert marker");
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}
