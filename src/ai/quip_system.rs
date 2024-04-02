use specs::{Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage};

use crate::{gamelog::Logger, MyTurn, Name, Quips, Viewshed};

pub struct QuipSystem;

impl<'a> System<'a> for QuipSystem {
    type SystemData = (
        WriteStorage<'a, Quips>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, MyTurn>,
        ReadExpect<'a, rltk::Point>,
        ReadStorage<'a, Viewshed>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut quips, names, turns, player_pos, viewsheds, mut rng) = data;

        for (quip, name, viewshed, _turn) in (&mut quips, &names, &viewsheds, &turns).join() {
            if !quip.available.is_empty()
                && viewshed.visible_tiles.contains(&player_pos)
                && rng.roll_dice(1, 6) == 1
            {
                let quip_index = if quip.available.len() == 1 {
                    0
                } else {
                    (rng.roll_dice(1, quip.available.len() as i32) - 1) as usize
                };

                Logger::new()
                    .yellow(&name.name)
                    .white("says")
                    .cyan(&quip.available[quip_index])
                    .log();
                quip.available.remove(quip_index);
            }
        }
    }
}
