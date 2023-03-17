use super::{Map, Position};
mod simple_map;
use simple_map::SimpleMapBuilder;
mod bsp_dungeon;
use bsp_dungeon::BspDungeonBuilder;
mod bsp_interior;
use bsp_interior::BspInteriorBuilder;
mod cellular_automata;
use cellular_automata::CellularAutomataBuilder;
mod drunkard;
use drunkard::DrunkardsWalkBuilder;
mod maze;
use maze::MazeBuilder;
mod dla;
use dla::DLABuilder;
mod common;
mod voronoi;
use specs::World;
use voronoi::VoronoiCellBuilder;
mod waveform_collapse;
use waveform_collapse::WaveformCollapseBuilder;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
    fn get_snapshot_history(&self) -> Vec<Map>;
    fn take_snapshot(&mut self);
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = rltk::RandomNumberGenerator::new();
    let builder = rng.roll_dice(1, 9);
    let mut result: Box<dyn MapBuilder> = match builder {
        1 => Box::new(SimpleMapBuilder::new(new_depth)),
        2 => Box::new(BspDungeonBuilder::new(new_depth)),
        3 => Box::new(BspInteriorBuilder::new(new_depth)),
        4 => Box::new(CellularAutomataBuilder::new(new_depth)),
        5 => Box::new(MazeBuilder::new(new_depth)),
        6 => Box::new(WaveformCollapseBuilder::test_map(new_depth)),
        7 => match rng.roll_dice(1, 3) {
            1 => Box::new(VoronoiCellBuilder::pythagoras(new_depth)),
            2 => Box::new(VoronoiCellBuilder::manhattan(new_depth)),
            _ => Box::new(VoronoiCellBuilder::chebyshev(new_depth)),
        },
        8 => match rng.roll_dice(1, 4) {
            1 => Box::new(DLABuilder::walk_inwards(new_depth)),
            2 => Box::new(DLABuilder::walk_outwards(new_depth)),
            3 => Box::new(DLABuilder::central_attractor(new_depth)),
            _ => Box::new(DLABuilder::insectoid(new_depth)),
        },
        _ => match rng.roll_dice(1, 5) {
            1 => Box::new(DrunkardsWalkBuilder::open_area(new_depth)),
            2 => Box::new(DrunkardsWalkBuilder::open_halls(new_depth)),
            3 => Box::new(DrunkardsWalkBuilder::fat_passages(new_depth)),
            4 => Box::new(DrunkardsWalkBuilder::fearful_symmetry(new_depth)),
            _ => Box::new(DrunkardsWalkBuilder::winding_passages(new_depth)),
        },
    };

    if rng.roll_dice(1, 3) == 1 {
        result = Box::new(WaveformCollapseBuilder::derived_map(new_depth, result));
    }

    result
}
