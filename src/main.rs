use animal_ai_system::AnimalAISystem;
use bystander_ai_system::BystanderAISystem;
use rltk::{GameState, Point, Rltk};
use specs::{
    prelude::*,
    saveload::{SimpleMarker, SimpleMarkerAllocator},
};
#[macro_use]
extern crate lazy_static;

pub mod map;
use map::{
    dungeon::{freeze_level_entities, level_transition, thaw_level_entities},
    *,
};
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
use map::dungeon::MasterDungeonMap;
mod animal_ai_system;
pub mod bystander_ai_system;
pub mod camera;
mod gamesystem;
mod hunger_system;
mod lighting_system;
pub mod map_builders;
mod particle_system;
mod random_table;
mod raws;
mod rex_assets;
mod saveload_system;
mod trigger_system;
use lighting_system::LightingSystem;

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
    PreviousLevel,
    GameOver,
    MagicMapReveal {
        row: i32,
    },
    MapGeneration,
    ShowCheatMenu,
}

pub struct State {
    pub ecs: World,
    mapgen_next_state: Option<RunState>,
    mapgen_history: Vec<Map>,
    mapgen_index: usize,
    mapgen_timer: f32,
}

impl State {
    fn generate_world_map(&mut self, new_depth: i32, offset: i32) {
        self.mapgen_index = 0;
        self.mapgen_timer = 0.0;
        self.mapgen_history.clear();
        let map_building_info = level_transition(&mut self.ecs, new_depth, offset);
        if let Some(history) = map_building_info {
            self.mapgen_history = history;
        } else {
            thaw_level_entities(&mut self.ecs);
        }
    }

    fn goto_level(&mut self, offset: i32) {
        freeze_level_entities(&mut self.ecs);

        // Build a new map and place the player
        let new_depth = self.ecs.fetch::<Map>().depth + offset;
        self.generate_world_map(new_depth, offset);

        // Notify the player
        let mut gamelog = self.ecs.fetch_mut::<gamelog::Gamelog>();
        gamelog.entries.push("You change level.".to_string());
    }

    fn game_over_cleanup(&mut self) {
        self.ecs.delete_all();

        // Spawn a new player
        {
            let player_entity = spawner::player(&mut self.ecs, Position { x: 0, y: 0 });
            let mut player_entity_writer = self.ecs.write_resource::<Entity>();
            *player_entity_writer = player_entity;
        }

        // Replace the world maps
        self.ecs.insert(MasterDungeonMap::new());

        // Build a new map and place the player
        self.generate_world_map(1, 0);
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

        // TODO(aalhendi): Run order... currently have to attack deer's *next* position.
        let mut animal_ai_system = AnimalAISystem;
        animal_ai_system.run_now(&self.ecs);

        let mut bystander_ai_system = BystanderAISystem;
        bystander_ai_system.run_now(&self.ecs);

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

        let mut lighting_system = LightingSystem;
        lighting_system.run_now(&self.ecs);

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
                self.goto_level(1);
                newrunstate = RunState::PreRun;
            }
            RunState::PreviousLevel => {
                self.goto_level(-1);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
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

            RunState::ShowCheatMenu => {
                let result = gui::show_cheat_mode(self, ctx);
                match result {
                    gui::CheatMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    gui::CheatMenuResult::NoResponse => {}
                    gui::CheatMenuResult::TeleportToExit => {
                        self.goto_level(1);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        newrunstate = RunState::MapGeneration;
                    }
                    gui::CheatMenuResult::MagicMapper => {
                        newrunstate = RunState::MagicMapReveal { row: 0 }
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
    let context = RltkBuilder::simple(80, 60)
        .unwrap()
        .with_title("Rust Roguelike !")
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
    gs.ecs.register::<Bystander>();
    gs.ecs.register::<Vendor>();
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
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
    gs.ecs.register::<MeleeWeapon>();
    gs.ecs.register::<Wearable>();
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
    gs.ecs.register::<Quips>();
    gs.ecs.register::<Attributes>();
    gs.ecs.register::<Skills>();
    gs.ecs.register::<Pools>();
    gs.ecs.register::<NaturalAttackDefense>();
    gs.ecs.register::<LootTable>();
    gs.ecs.register::<Carnivore>();
    gs.ecs.register::<Herbivore>();
    gs.ecs.register::<OtherLevelPosition>();
    gs.ecs.register::<DMSerializationHelper>();
    gs.ecs.register::<LightSource>();

    gs.ecs.insert(SimpleMarkerAllocator::<IsSerialized>::new());
    raws::load_raws();

    // Resource Insertion
    let player_entity = spawner::player(&mut gs.ecs, Position { x: 0, y: 0 });
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(rex_assets::RexAssets::new());
    gs.ecs.insert(Map::new(1, 64, 64, "New Map"));
    gs.ecs.insert(MasterDungeonMap::new());
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

    gs.generate_world_map(1, 0);

    rltk::main_loop(context, gs)
}
