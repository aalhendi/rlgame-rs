use rltk::{GameState, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;
use specs_derive::Component;

const WINDOW_HEIGHT: i32 = 50;
const WINDOW_WIDTH: i32 = 80;
const SPAWN_X: i32 = 40;
const SPAWN_Y: i32 = 25;

// --- Components Start ---
#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: rltk::FontCharType,
    fg: RGB,
    bg: RGB,
}

#[derive(Component, Debug)]
struct Player {}

#[derive(Component)]
struct LeftMover {}
// --- Components End ---

// --- Map Start ---
#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall,
    Floor,
}

fn xy_idx(x: i32, y: i32) -> usize {
    (y * WINDOW_WIDTH + x) as usize
}

fn new_map() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; (WINDOW_WIDTH * WINDOW_HEIGHT) as usize];

    // Setting window boundaries as walls
    for x in 0..WINDOW_WIDTH {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, WINDOW_HEIGHT - 1)] = TileType::Wall;
    }
    for y in 0..WINDOW_HEIGHT {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(WINDOW_WIDTH - 1, y)] = TileType::Wall;
    }

    // Random Walls on ~10% of tiles via thread-local rng
    let mut rng = rltk::RandomNumberGenerator::new();
    let spawn_idx = xy_idx(SPAWN_X, SPAWN_Y);
    for _ in 0..400 {
        let x = rng.roll_dice(1, WINDOW_WIDTH - 1);
        let y = rng.roll_dice(1, WINDOW_HEIGHT - 1);
        let idx = xy_idx(x, y);
        if idx != spawn_idx {
            map[idx] = TileType::Wall;
        }
    }

    map
}

fn draw_map(map: &[TileType], ctx: &mut Rltk) {
    let mut y = 0;
    let mut x = 0;
    for tile in map.iter() {
        match tile {
            TileType::Floor => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.5, 0.5, 0.5),
                    RGB::named(rltk::BLACK),
                    rltk::to_cp437('.'),
                );
            }
            TileType::Wall => {
                ctx.set(
                    x,
                    y,
                    RGB::from_f32(0.0, 1.0, 0.0),
                    RGB::named(rltk::BLACK),
                    rltk::to_cp437('#'),
                );
            }
        }

        // iter coordinates as well
        x += 1;
        if x > WINDOW_WIDTH - 1 {
            x = 0;
            y += 1;
        }
    }
}
// --- Map End ---

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

fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let map = ecs.fetch::<Vec<TileType>>();

    for (_player, pos) in (&mut players, &mut positions).join() {
        let dest_idx = xy_idx(pos.x + delta_x, pos.y + delta_y);
        if map[dest_idx] != TileType::Wall {
            pos.x = (pos.x + delta_x).clamp(0, WINDOW_WIDTH - 1);
            pos.y = (pos.y + delta_y).clamp(0, WINDOW_HEIGHT - 1);
        }
    }
}

fn player_input(gs: &mut State, ctx: &mut Rltk) {
    if let Some(key) = ctx.key {
        match key {
            VirtualKeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            VirtualKeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            VirtualKeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            VirtualKeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            _ => {}
        };
    }
}

// --- State Start ---
struct State {
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
