use super::Map;
mod simple_map;
use simple_map::SimpleMapBuilder;
mod common;

trait MapBuilder {
    fn build(new_depth: i32) -> Map;
}

pub fn build_random_map(new_depth: i32) -> Map {
    SimpleMapBuilder::build(new_depth)
}
