use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

pub mod map;
use map::*;
pub mod components;
use components::*;
pub mod player;
use player::*;
pub mod rect;
use rect::Rect;
use visibility_system::VisibilitySystem;
pub mod visibility_system;

pub const WINDOW_HEIGHT: i32 = 50;
pub const WINDOW_WIDTH: i32 = 80;
const PLAYER_VIEW_RANGE: i32 = 8;

// --- State Start ---
pub struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        draw_map(&self.ecs, ctx);

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph);
        }
    }
}
// --- State End ---

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut gs = State { ecs: World::new() };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<LeftMover>();

    let map = Map::new_map_rooms_and_corridors();
    let player_pos = map.rooms[0].center();
    gs.ecs.insert(map);

    gs.ecs
        .create_entity()
        .with(player_pos)
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .with(Viewshed {
            visible_tiles: Vec::new(),
            range: PLAYER_VIEW_RANGE,
        })
        .build();

    rltk::main_loop(context, gs)
}
