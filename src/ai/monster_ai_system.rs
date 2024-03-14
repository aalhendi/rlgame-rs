use rltk::Point;
use specs::prelude::*;

use crate::{EntityMoved, Map, Monster, MyTurn, Position, RunState, Viewshed, WantsToMelee};

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
    WriteStorage<'a, EntityMoved>,
    ReadStorage<'a, MyTurn>,
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
            _runstate,
            mut entity_moved,
            turns,
        ) = data;

        for (entity, viewshed, _monster, pos, _turn) in
            (&entities, &mut viewshed, &monster, &mut position, &turns).join()
        {
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
                    let (x, y) = map.idx_xy(path.steps[1]);
                    pos.x = x;
                    pos.y = y;
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
