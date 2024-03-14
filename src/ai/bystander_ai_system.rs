use specs::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{
    components::{Bystander, EntityMoved, Position, Viewshed},
    map::Map,
    MyTurn, RunState,
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
        ReadStorage<'a, MyTurn>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            _runstate,
            entities,
            mut viewsheds,
            bystanders,
            mut positions,
            mut entity_moved,
            mut rng,
            turns,
        ) = data;

        for (entity, viewshed, _bystander, pos, _turn) in (
            &entities,
            &mut viewsheds,
            &bystanders,
            &mut positions,
            &turns,
        )
            .join()
        {
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
