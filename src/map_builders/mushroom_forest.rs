use super::{area_ending_point::{AreaEndingPosition, XEnd, YEnd}, area_starting_points::{AreaStartingPosition, XStart, YStart}, cellular_automata::CellularAutomataBuilder, cull_unreachable::CullUnreachable, prefab_builder::{prefab_sections::UNDERGROUND_FORT, PrefabBuilder}, voronoi_spawning::VoronoiSpawning, waveform_collapse::WaveformCollapseBuilder, BuilderChain};

pub fn mushroom_entrance(
    new_depth: i32,
    _rng: &mut rltk::RandomNumberGenerator,
    width: i32,
    height: i32,
) -> BuilderChain {
    let mut chain = BuilderChain::new(new_depth, width, height, "Into The Mushroom Grove");
    chain.start_with(CellularAutomataBuilder::new());
    chain.with(WaveformCollapseBuilder::new());
    chain.with(AreaStartingPosition::new(XStart::Center, YStart::Center));
    chain.with(CullUnreachable::new());
    chain.with(AreaStartingPosition::new(XStart::Right, YStart::Center));
    chain.with(AreaEndingPosition::new(XEnd::Left, YEnd::Center));
    chain.with(VoronoiSpawning::new());
    chain.with(PrefabBuilder::sectional(UNDERGROUND_FORT));
    chain
}
