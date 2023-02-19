use rltk::{GameState, Point, Rltk, RGB};
use specs::prelude::*;

pub mod map;
use map::*;
pub mod components;
use components::*;
pub mod player;
use player::*;
pub mod rect;
use rect::Rect;
pub mod visibility_system;
use visibility_system::VisibilitySystem;
pub mod monster_ai_system;
use monster_ai_system::MonsterAI;
pub mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
pub mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
pub mod damage_system;
use damage_system::DamageSystem;

pub const WINDOW_HEIGHT: i32 = 50;
pub const WINDOW_WIDTH: i32 = 80;
const PLAYER_VIEW_RANGE: i32 = 8;

// --- State Start ---
#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    Paused,
    Running,
}

pub struct State {
    pub ecs: World,
    pub runstate: RunState,
}

impl State {
    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI {};
        mob.run_now(&self.ecs);

        let mut mapindex = MapIndexingSystem {};
        mapindex.run_now(&self.ecs);

        let mut melee_combat_system = MeleeCombatSystem {};
        melee_combat_system.run_now(&self.ecs);

        let mut damage_system = DamageSystem {};
        damage_system.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();

        if self.runstate == RunState::Running {
            self.run_systems();
            damage_system::delete_the_dead(&mut self.ecs);
            self.runstate = RunState::Paused;
        } else {
            self.runstate = player_input(self, ctx);
        }

        draw_map(&self.ecs, ctx);

        let map = self.ecs.fetch::<Map>();
        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            if map.visible_tiles[idx] {
                ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
            }
        }
    }
}
// --- State End ---

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;

    let mut gs = State {
        ecs: World::new(),
        runstate: RunState::Running,
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<Player>();
    gs.ecs.register::<Viewshed>();
    gs.ecs.register::<Monster>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<CombatStats>();
    gs.ecs.register::<WantsToMelee>();
    gs.ecs.register::<SufferDamage>();

    let map = Map::new_map_rooms_and_corridors();
    let player_pos = map.rooms[0].center();

    let mut rng = rltk::RandomNumberGenerator::new();
    for (idx, room) in map.rooms.iter().skip(1).enumerate() {
        let pos = room.center();
        let name: String;
        let glyph: rltk::FontCharType;
        let roll = rng.roll_dice(1, 2);

        match roll {
            1 => {
                glyph = rltk::to_cp437('g');
                name = "Goblin".to_string();
            }
            _ => {
                glyph = rltk::to_cp437('o');
                name = "Orc".to_string();
            }
        };

        gs.ecs
            .create_entity()
            .with(pos)
            .with(Renderable {
                glyph,
                fg: RGB::named(rltk::RED),
                bg: RGB::named(rltk::BLACK),
            })
            .with(Monster {})
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(Name {
                name: format!("{name} {idx}", name = &name),
            })
            .with(BlocksTile {})
            .with(CombatStats {
                max_hp: 16,
                hp: 16,
                defense: 1,
                power: 4,
            })
            .build();
    }

    let player_entity = gs
        .ecs
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
            dirty: true,
        })
        .with(Name {
            name: "Player".to_string(),
        })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .build();

    gs.ecs.insert(Point::new(player_pos.x, player_pos.y));
    gs.ecs.insert(map);
    gs.ecs.insert(player_entity);
    rltk::main_loop(context, gs)
}
