use super::{Map, Position};
mod simple_map;
use simple_map::SimpleMapBuilder;
mod common;
use specs::World;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    // Not really random when there is only one map type...
    Box::new(SimpleMapBuilder::new(new_depth))
}
