use rltk::Point;
use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    components::{Bystander, EntityMoved, Name, Position, Quips, Viewshed},
    gamelog::Gamelog,
    map::Map,
    RunState,
};

pub struct BystanderAISystem;

impl<'a> System<'a> for BystanderAISystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        ReadStorage<'a, Bystander>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, EntityMoved>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadExpect<'a, Point>,
        WriteExpect<'a, Gamelog>,
        WriteStorage<'a, Quips>,
        ReadStorage<'a, Name>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            runstate,
            entities,
            mut viewsheds,
            bystanders,
            mut positions,
            mut entity_moved,
            mut rng,
            player_pos,
            mut gamelog,
            mut all_quips,
            names,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }

        for (entity, viewshed, _bystander, pos) in
            (&entities, &mut viewsheds, &bystanders, &mut positions).join()
        {
            if let Some(quips) = all_quips.get_mut(entity) {
                if !quips.available.is_empty()
                    && viewshed.visible_tiles.contains(&player_pos)
                    && rng.roll_dice(1, 6) == 1
                {
                    let quip_idx = if quips.available.len() == 1 {
                        0
                    } else {
                        (rng.roll_dice(1, quips.available.len() as i32) - 1) as usize
                    };
                    gamelog.entries.push(format!(
                        "{name} says \"{quip}\"",
                        name = names.get(entity).unwrap().name,
                        quip = quips.available[quip_idx]
                    ))
                }
            }
            // Try to move randomly
            let mut x = pos.x;
            let mut y = pos.y;
            let move_roll = rng.roll_dice(1, 5);
            match move_roll {
                1 => x -= 1,
                2 => x += 1,
                3 => y -= 1,
                4 => y += 1,
                _ => {}
            }

            // Check bounds
            // TODO: Abstract bounds checking to method. Used in multiple places.
            if x > 0 && x < map.width - 1 && y > 0 && y < map.height - 1 {
                let dest_idx = map.xy_idx(x, y);
                // If new pos is not blocked, clear old pos, set new pos.
                if !map.blocked[dest_idx] {
                    let idx = map.xy_idx(pos.x, pos.y);
                    map.blocked[idx] = false;
                    pos.x = x;
                    pos.y = y;
                    entity_moved
                        .insert(entity, EntityMoved {})
                        .expect("Unable to insert marker");
                    map.blocked[dest_idx] = true;
                    viewshed.dirty = true;
                }
            }
        }
    }
}
