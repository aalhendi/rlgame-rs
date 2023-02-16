use rltk::RGB;
use specs::prelude::*;
use specs_derive::Component;

#[derive(Component)]
// used in viewshed instead of rltk::Point which is the same struct
// with some exrta derives listed below. add as needed
// #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// #[derive(Eq, PartialEq, Copy, Clone, Debug, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Component)]
pub struct Renderable {
    pub glyph: rltk::FontCharType,
    pub fg: RGB,
    pub bg: RGB,
}

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component)]
pub struct LeftMover {}

#[derive(Component)]
pub struct Viewshed {
    pub visible_tiles: Vec<Position>,
    pub range: i32,
}
