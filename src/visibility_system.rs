use super::{Position, Viewshed};
use specs::prelude::*;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (WriteStorage<'a, Viewshed>, WriteStorage<'a, Position>);

    fn run(&mut self, (mut viewshed, pos): Self::SystemData){
        // tuple and join used here to ensure only entities with BOTH viewshed and positon get
        // called
        for (viewshed, pos) in (&mut viewshed, &pos).join() {
            continue;
        }
    }
}
