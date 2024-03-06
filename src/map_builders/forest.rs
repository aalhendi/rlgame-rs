use super::{
    area_starting_points::{AreaStartingPosition, XStart, YStart},
    cellular_automata::CellularAutomataBuilder,
    cull_unreachable::CullUnreachable,
    voronoi_spawning::VoronoiSpawning,
    yellow_brick_road::YellowBrickRoad,
    BuilderChain,
};

pub fn forest_builder(
    new_depth: i32,
    _rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into the Woods");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::Left, YStart::Center));
    chain.with(VoronoiSpawning::new());
    chain.with(YellowBrickRoad::new()); // Exit

    chain
}
