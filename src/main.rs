use rltk::{GameState, Rltk, RGB};
use specs::prelude::*;

pub mod map;
use map::*;
pub mod components;
use components::*;
pub mod player;
use player::*;


pub const WINDOW_HEIGHT: i32 = 50;
pub const WINDOW_WIDTH: i32 = 80;
pub const SPAWN_X: i32 = 40;
pub const SPAWN_Y: i32 = 25;

// --- Systems Start ---
struct LeftWalker {}

impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (leftmovers, mut pos): Self::SystemData) {
        for (_leftmover, pos) in (&leftmovers, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 {
                pos.x = WINDOW_WIDTH;
            }
        }
    }
}
// --- Systems End ---

// --- State Start ---
pub struct State {
    ecs: World,
}

impl State {
    fn run_systems(&mut self) {
        let mut lw = LeftWalker {};
        lw.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        player_input(self, ctx);
        self.run_systems();

        let map = self.ecs.fetch::<Vec<TileType>>();
        draw_map(&map, ctx);

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

    gs.ecs.insert(new_map());

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<LeftMover>();

    gs.ecs
        .create_entity()
        .with(Position {
            x: SPAWN_X,
            y: SPAWN_Y,
        })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(Player {})
        .build();

    rltk::main_loop(context, gs)
}
