use super::{Map, Monster, Name, Position, Viewshed};
use rltk::{console, Point};
use specs::prelude::*;

pub struct MonsterAI {}

type MonsterAIData<'a> = (
    WriteStorage<'a, Viewshed>,
    ReadExpect<'a, Point>,
    ReadStorage<'a, Monster>,
    ReadStorage<'a, Name>,
    WriteStorage<'a, Position>,
    ReadExpect<'a, Map>,
);
impl<'a> System<'a> for MonsterAI {
    type SystemData = MonsterAIData<'a>;

    fn run(&mut self, data: Self::SystemData) {
        let (mut viewshed, player_pos, monster, name, mut position, map) = data;

        for (mut viewshed, _monster, name, mut pos) in
            (&mut viewshed, &monster, &name, &mut position).join()
        {
            if viewshed.visible_tiles.contains(&*player_pos) {
                console::log(&format!(
                    "{mon_name} considers their own existence",
                    mon_name = name.name
                ));
                let path = rltk::a_star_search(
                    map.xy_idx(pos.x, pos.y) as i32,
                    map.xy_idx(player_pos.x, player_pos.y) as i32,
                    &*map,
                );
                if path.success && path.steps.len() > 1 {
                    pos.x = path.steps[1] as i32 % map.width;
                    pos.y = path.steps[1] as i32 / map.width;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
