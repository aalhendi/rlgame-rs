use rltk::{GameState, Point, Rltk};
use specs::{
    prelude::*,
    saveload::{SimpleMarker, SimpleMarkerAllocator},
};

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
mod gamelog;
mod gui;
pub mod inventory_system;
pub mod spawner;
use inventory_system::{ItemCollectionSystem, ItemDropSystem, ItemRemoveSystem, ItemUseSystem};
pub mod camera;
mod hunger_system;
pub mod map_builders;
mod particle_system;
mod random_table;
mod rex_assets;
mod saveload_system;
mod trigger_system;

const SHOW_MAPGEN_VISUALIZER: bool = true;

// --- State Start ---
#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowRemoveItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: gui::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
    GameOver,
    MagicMapReveal {
        row: i32,
    },
    MapGeneration,
}

pub struct State {
    pub ecs: World,
    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl State {
    fn generate_world_map(&mut self, new_depth: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let mut rng = self.ecs.write_resource::<rltk::RandomNumberGenerator>();
        let mut builder = map_builders::random_builder(new_depth, &mut rng, 80, 50);
        builder.build_map(&mut rng);
        self.mapgen_history = builder.build_data.history.clone();
        let player_start = {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            *worldmap_resource = builder.build_data.map.clone();
            *builder.build_data.starting_position.as_mut().unwrap()
        };

        // Spawn bad guys
        std::mem::drop(rng);
        builder.spawn_entities(&mut self.ecs);

        // Place the player and update resources
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_start.x, player_start.y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        if let Some(player_pos_comp) = position_components.get_mut(*player_entity) {
            *player_pos_comp = player_start;
        }

        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        if let Some(player_vs) = viewshed_components.get_mut(*player_entity) {
            player_vs.dirty = true;
        }
    }

    fn goto_next_level(&mut self) {
        let to_delete = self.entities_to_remove_on_level_change();
        for e in to_delete {
            self.ecs.delete_entity(e).expect("Unable to delte entity");
        }

        // Build a new map and place the player
        let current_depth = {
            let worldmap_resource = self.ecs.fetch::<Map>();
            worldmap_resource.depth
        };
        self.generate_world_map(current_depth + 1);

        // Notify the player and give them some health
        let player_entity = self.ecs.fetch::<Entity>();
        let mut gamelog = self.ecs.fetch_mut::<gamelog::Gamelog>();
        let mut combat_stats_store = self.ecs.write_storage::<CombatStats>();

        gamelog
            .entries
            .push("You descend to the next level, and take a moment to heal.".to_string());

        if let Some(player_stats) = combat_stats_store.get_mut(*player_entity) {
            player_stats.hp = i32::max(player_stats.hp, player_stats.max_hp / 2);
        }
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let player_entity = self.ecs.fetch::<Entity>();
        let equipped = self.ecs.read_storage::<Equipped>();

        let mut to_delete: Vec<Entity> = Vec::new();

        for e in entities.join() {
            let is_in_player_backpack = match backpack.get(e) {
                None => false,
                Some(e) => e.owner == *player_entity,
            };
            let is_equipped = match equipped.get(e) {
                None => false,
                Some(e) => e.owner == *player_entity,
            };

            // Don't delete player or their items
            if is_in_player_backpack || player.get(e).is_some() || is_equipped {
                continue;
            } else {
                to_delete.push(e);
            }
        }
        to_delete
    }

    fn game_over_cleanup(&mut self) {
        self.ecs.delete_all();

        // Spawn a new player
        {
            let player_entity = spawner::player(&mut self.ecs, Position { x: 0, y: 0 });
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }

        // Build a new map and place the player
        self.generate_world_map(1);
    }

    fn run_systems(&mut self) {
        let mut vis = VisibilitySystem;
        vis.run_now(&self.ecs);

        let mut mob = MonsterAI;
        mob.run_now(&self.ecs);

        let mut trigger_system = trigger_system::TriggerSystem;
        trigger_system.run_now(&self.ecs);

        let mut mapindex = MapIndexingSystem;
        mapindex.run_now(&self.ecs);

        let mut melee_combat_system = MeleeCombatSystem;
        melee_combat_system.run_now(&self.ecs);

        let mut damage_system = DamageSystem;
        damage_system.run_now(&self.ecs);

        let mut item_collection_system = ItemCollectionSystem;
        item_collection_system.run_now(&self.ecs);

        let mut item_use_system = ItemUseSystem;
        item_use_system.run_now(&self.ecs);

        let mut item_drop_system = ItemDropSystem;
        item_drop_system.run_now(&self.ecs);

        let mut item_remove_system = ItemRemoveSystem;
        item_remove_system.run_now(&self.ecs);

        let mut hunger_system = hunger_system::HungerSystem;
        hunger_system.run_now(&self.ecs);

        let mut particle_system = particle_system::ParticleSpawnSystem;
        particle_system.run_now(&self.ecs);

        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        let mut newrunstate = { *self.ecs.fetch::<RunState>() };
        ctx.cls();
        particle_system::cull_dead_particles(&mut self.ecs, ctx);

        // Either draw Main Menu or draw map
        match newrunstate {
            RunState::MainMenu { .. } => {}
            RunState::GameOver => {}
            _ => {
                camera::render_camera(&self.ecs, ctx);
                gui::draw_ui(&self.ecs, ctx);
            }
        }

        // Game states
        match newrunstate {
            RunState::PreRun => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems();
                self.ecs.maintain();
                match *self.ecs.fetch::<RunState>() {
                    RunState::MagicMapReveal { .. } => {
                        newrunstate = RunState::MagicMapReveal { row: 0 }
                    }
                    _ => newrunstate = RunState::MonsterTurn,
                }
            }
            RunState::MonsterTurn => {
                self.run_systems();
                self.ecs.maintain();
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowDropItem => {
                let (item_menu_result, item_entity) = gui::drop_item_menu(self, ctx);
                match item_menu_result {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = item_entity.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowRemoveItem => {
                let (item_menu_result, item_entity) = gui::remove_item_menu(self, ctx);
                match item_menu_result {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = item_entity.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToRemoveItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowInventory => {
                let (item_menu_result, item_entity) = gui::show_inventory(self, ctx);
                match item_menu_result {
                    gui::ItemMenuResult::Cancel => {
                        newrunstate = RunState::AwaitingInput;
                    }
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let item_entity = item_entity.unwrap();
                        if let Some(item) = self.ecs.read_storage::<Ranged>().get(item_entity) {
                            newrunstate = RunState::ShowTargeting {
                                range: item.range,
                                item: item_entity,
                            }
                        } else {
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToUseItem {
                                        item: item_entity,
                                        target: None,
                                    },
                                )
                                .expect("Unable to insert intent");
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let (item_menu_result, item_entity) = gui::ranged_target(self, ctx, range);
                match item_menu_result {
                    gui::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::ItemMenuResult::NoResponse => {}
                    gui::ItemMenuResult::Selected => {
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToUseItem {
                                    item,
                                    target: item_entity,
                                },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let main_menu_result = gui::main_menu(self, ctx);
                match main_menu_result {
                    gui::MainMenuResult::NoSelection { highlighted } => {
                        newrunstate = RunState::MainMenu {
                            menu_selection: highlighted,
                        }
                    }
                    gui::MainMenuResult::Selected { highlighted } => match highlighted {
                        gui::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                        gui::MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.ecs);
                            newrunstate = RunState::AwaitingInput;
                            saveload_system::delete_save();
                        }
                        gui::MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu {
                    menu_selection: gui::MainMenuSelection::Quit,
                };
            }
            RunState::NextLevel => {
                self.goto_next_level();
                newrunstate = RunState::PreRun;
            }
            RunState::GameOver => {
                let game_over_result = gui::game_over(ctx);
                match game_over_result {
                    gui::GameOverResult::NoSelection => {}
                    gui::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        newrunstate = RunState::MainMenu {
                            menu_selection: gui::MainMenuSelection::NewGame,
                        };
                    }
                }
            }
            RunState::MagicMapReveal { row } => {
                let mut map = self.ecs.fetch_mut::<Map>();
                for x in 0..map.width {
                    let idx = map.xy_idx(x, row);
                    map.revealed_tiles[idx] = true;
                }
                if row == map.height - 1 {
                    newrunstate = RunState::MonsterTurn;
                } else {
                    newrunstate = RunState::MagicMapReveal { row: row + 1 };
                }
            }
            RunState::MapGeneration => {
                if !SHOW_MAPGEN_VISUALIZER {
                    newrunstate = self.mapgen_next_state.unwrap();
                }
                ctx.cls();
                if self.mapgen_index < self.mapgen_history.len() {
                    camera::render_debug_map(&self.mapgen_history[self.mapgen_index], ctx);
                }

                self.mapgen_timer += ctx.frame_time_ms;
                if self.mapgen_timer > 300.0 {
                    self.mapgen_timer = 0.0;
                    self.mapgen_index += 1;
                    if self.mapgen_index >= self.mapgen_history.len() {
                        newrunstate = self.mapgen_next_state.unwrap();
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);
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
        mapgen_next_state: Some(RunState::MainMenu {
            menu_selection: gui::MainMenuSelection::NewGame,
        }),
        mapgen_index: 0,
        mapgen_history: Vec::new(),
        mapgen_timer: 0.0,
    };

    // Component registration
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
    gs.ecs.register::<InflictsDamage>();
    gs.ecs.register::<Confusion>();
    gs.ecs.register::<AreaOfEffect>();
    gs.ecs.register::<Consumable>();
    gs.ecs.register::<Ranged>();
    gs.ecs.register::<Item>();
    gs.ecs.register::<ProvidesHealing>();
    gs.ecs.register::<InBackpack>();
    gs.ecs.register::<WantsToPickupItem>();
    gs.ecs.register::<WantsToUseItem>();
    gs.ecs.register::<WantsToDropItem>();
    gs.ecs.register::<WantsToRemoveItem>();
    gs.ecs.register::<Equippable>();
    gs.ecs.register::<Equipped>();
    gs.ecs.register::<SimpleMarker<IsSerialized>>();
    gs.ecs.register::<SerializationHelper>();
    gs.ecs.register::<MeleePowerBonus>();
    gs.ecs.register::<DefenseBonus>();
    gs.ecs.register::<ParticleLifetime>();
    gs.ecs.register::<HungerClock>();
    gs.ecs.register::<ProvidesFood>();
    gs.ecs.register::<MagicMapper>();
    gs.ecs.register::<Hidden>();
    gs.ecs.register::<EntryTrigger>();
    gs.ecs.register::<EntityMoved>();
    gs.ecs.register::<SingleActivation>();
    gs.ecs.register::<BlocksVisibility>();
    gs.ecs.register::<Door>();

    gs.ecs.insert(SimpleMarkerAllocator::<IsSerialized>::new());

    // Resource Insertion
    let player_entity = spawner::player(&mut gs.ecs, Position { x: 0, y: 0 });
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(rex_assets::RexAssets::new());
    gs.ecs.insert(Map::new(1, 64, 64));
    gs.ecs.insert(Point::new(0, 0));
    gs.ecs.insert(player_entity);
    if SHOW_MAPGEN_VISUALIZER {
        gs.ecs.insert(RunState::MapGeneration {});
    } else {
        gs.ecs.insert(RunState::MainMenu {
            menu_selection: gui::MainMenuSelection::NewGame,
        });
    }
    gs.ecs.insert(gamelog::Gamelog {
        entries: vec!["Welcome to Rusty Rougelike".to_string()],
    });
    gs.ecs.insert(particle_system::ParticleBuilder::new());

    gs.generate_world_map(1);

    rltk::main_loop(context, gs)
}
