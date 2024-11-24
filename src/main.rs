use ai::{
    adjacent_ai_system::AdjacentAI, approach_ai_system::ApproachAI, chase_ai_system::ChaseAI,
    default_move_ai::DefaultMoveAI, initiative_system::InitiativeSystem, quip_system::QuipSystem,
    turn_status_system::TurnStatusSystem, visible_ai_system::VisibleAI,
};
use encumbrance_system::EncumbranceSystem;
use gamelog::Logger;
use gui::menu::{
    cheat::{show_cheat_mode, CheatMenuResult},
    game_over::{game_over, GameOverResult},
    identify::identify_menu,
    main_menu::{main_menu, MainMenuResult, MainMenuSelection},
    remove_curse::remove_curse_menu,
    vendor::{show_vendor_menu, VendorResult},
    ItemMenuResult,
};
use movement_system::MovementSystem;
use ranged_combat_system::RangedCombatSystem;
use raws::{
    rawsmaster::{spawn_named_item, SpawnType},
    RAWS,
};
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
pub mod encumbrance_system;
use components::*;
pub mod player;
use player::*;
pub mod rect;
use rect::Rect;
pub mod visibility_system;
use trigger_system::TriggerSystem;
use visibility_system::VisibilitySystem;
pub mod map_indexing_system;
use map_indexing_system::MapIndexingSystem;
pub mod melee_combat_system;
use melee_combat_system::MeleeCombatSystem;
pub mod damage_system;
mod gamelog;
mod gui;
pub mod inventory_system;
pub mod spawner;
use inventory_system::identification_system::ItemIdentificationSystem;
use inventory_system::remove_system::ItemRemoveSystem;
use inventory_system::use_system::ItemUseSystem;
use inventory_system::{collection_system::ItemCollectionSystem, use_equip::ItemEquipOnUse};
use inventory_system::{drop_system::ItemDropSystem, use_system::SpellUseSystem};
use map::dungeon::MasterDungeonMap;
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
mod ai;
mod effects;
mod movement_system;
mod ranged_combat_system;
pub mod spatial;

const SHOW_MAPGEN_VISUALIZER: bool = false;
const SHOW_FPS: bool = true;

#[derive(PartialEq, Copy, Clone)]
pub enum VendorMode {
    Buy,
    Sell,
}

// --- State Start ---
#[derive(PartialEq, Clone, Copy)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    Ticking,
    ShowInventory,
    ShowDropItem,
    ShowRemoveItem,
    ShowTargeting { range: i32, item: Entity },
    MainMenu { menu_selection: MainMenuSelection },
    SaveGame,
    NextLevel,
    PreviousLevel,
    TownPortal,
    GameOver,
    MagicMapReveal { row: i32 },
    MapGeneration,
    ShowCheatMenu,
    ShowVendor { vendor: Entity, mode: VendorMode },
    TeleportingToOtherLevel { x: i32, y: i32, depth: i32 },
    ShowRemoveCurse,
    ShowIdentify,
}

impl RunState {
    pub fn buy_vendor(vendor: Entity) -> Self {
        Self::ShowVendor {
            vendor,
            mode: VendorMode::Buy,
        }
    }

    pub fn sell_vendor(vendor: Entity) -> Self {
        Self::ShowVendor {
            vendor,
            mode: VendorMode::Sell,
        }
    }
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
        Logger::new().white("You change level.");
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
        let mut mapindex = MapIndexingSystem;
        mapindex.run_now(&self.ecs);

        let mut vis = VisibilitySystem;
        vis.run_now(&self.ecs);

        let mut encumbrance_system = EncumbranceSystem;
        encumbrance_system.run_now(&self.ecs);

        let mut initiative_system = InitiativeSystem;
        initiative_system.run_now(&self.ecs);

        let mut turn_status_system = TurnStatusSystem;
        turn_status_system.run_now(&self.ecs);

        let mut quip_system = QuipSystem;
        quip_system.run_now(&self.ecs);

        let mut adjecent_ai_system = AdjacentAI;
        adjecent_ai_system.run_now(&self.ecs);

        let mut visibile_ai_system = VisibleAI;
        visibile_ai_system.run_now(&self.ecs);

        let mut approach_ai_system = ApproachAI;
        approach_ai_system.run_now(&self.ecs);

        let mut chase_ai_system = ChaseAI;
        chase_ai_system.run_now(&self.ecs);

        let mut default_move_ai = DefaultMoveAI;
        default_move_ai.run_now(&self.ecs);

        let mut movement_system = MovementSystem;
        movement_system.run_now(&self.ecs);

        let mut trigger_system = TriggerSystem;
        trigger_system.run_now(&self.ecs);

        let mut melee_combat_system = MeleeCombatSystem;
        melee_combat_system.run_now(&self.ecs);

        let mut ranged_combat_system = RangedCombatSystem;
        ranged_combat_system.run_now(&self.ecs);

        let mut item_collection_system = ItemCollectionSystem;
        item_collection_system.run_now(&self.ecs);

        let mut item_equip_use_system = ItemEquipOnUse;
        item_equip_use_system.run_now(&self.ecs);

        let mut item_use_system = ItemUseSystem;
        item_use_system.run_now(&self.ecs);

        let mut spell_use_system = SpellUseSystem;
        spell_use_system.run_now(&self.ecs);

        let mut item_identification_system = ItemIdentificationSystem;
        item_identification_system.run_now(&self.ecs);

        let mut item_drop_system = ItemDropSystem;
        item_drop_system.run_now(&self.ecs);

        let mut item_remove_system = ItemRemoveSystem;
        item_remove_system.run_now(&self.ecs);

        let mut hunger_system = hunger_system::HungerSystem;
        hunger_system.run_now(&self.ecs);

        effects::run_effects_queue(&mut self.ecs);

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
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_active_console(0);
        ctx.cls();
        particle_system::update_particles(&mut self.ecs, ctx);

        // Either draw Main Menu or draw map
        match newrunstate {
            RunState::MainMenu { .. } => {}
            RunState::GameOver => {}
            _ => {
                camera::render_camera(&self.ecs, ctx);
                gui::hud::draw_ui(&self.ecs, ctx);
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
                if newrunstate != RunState::AwaitingInput {
                    gamelog::events::record_event("Turn", 1);
                }
            }
            RunState::Ticking => {
                let mut should_change_target = false;

                // runs all initiative cycles until it's the player's turn
                while newrunstate == RunState::Ticking {
                    self.run_systems();
                    self.ecs.maintain();
                    match *self.ecs.fetch::<RunState>() {
                        RunState::AwaitingInput => {
                            newrunstate = RunState::AwaitingInput;
                            should_change_target = true;
                        }
                        RunState::TownPortal => newrunstate = RunState::TownPortal,
                        RunState::ShowRemoveCurse => newrunstate = RunState::ShowRemoveCurse,
                        RunState::ShowIdentify => newrunstate = RunState::ShowIdentify,
                        RunState::TeleportingToOtherLevel { x, y, depth } => {
                            newrunstate = RunState::TeleportingToOtherLevel { x, y, depth }
                        }
                        RunState::MagicMapReveal { .. } => {
                            newrunstate = RunState::MagicMapReveal { row: 0 }
                        }
                        _ => newrunstate = RunState::Ticking,
                    }
                }
                if should_change_target {
                    player::end_turn_targeting(&mut self.ecs);
                }
            }
            RunState::ShowDropItem => {
                let (item_menu_result, item_entity) = gui::menu::drop_item_menu(self, ctx);
                match item_menu_result {
                    ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
                        let item_entity = item_entity.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToDropItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowRemoveItem => {
                let (item_menu_result, item_entity) = gui::menu::remove_item_menu(self, ctx);
                match item_menu_result {
                    ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
                        let item_entity = item_entity.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToRemoveItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowInventory => {
                let (item_menu_result, item_entity) = gui::menu::show_inventory(self, ctx);
                match item_menu_result {
                    ItemMenuResult::Cancel => {
                        newrunstate = RunState::AwaitingInput;
                    }
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
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
                            newrunstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let (item_menu_result, item_entity) =
                    gui::menu::ranged_target::ranged_target(self, ctx, range);
                match item_menu_result {
                    ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
                        if self.ecs.read_storage::<SpellTemplate>().get(item).is_some() {
                            let mut intent = self.ecs.write_storage::<WantsToCastSpell>();
                            intent
                                .insert(
                                    *self.ecs.fetch::<Entity>(),
                                    WantsToCastSpell {
                                        spell: item,
                                        target: item_entity,
                                    },
                                )
                                .expect("Unable to insert intent");
                            newrunstate = RunState::Ticking;
                        } else {
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
                            newrunstate = RunState::Ticking;
                        }
                    }
                }
            }
            RunState::MainMenu { .. } => {
                let main_menu_result = main_menu(self, ctx);
                match main_menu_result {
                    MainMenuResult::NoSelection { highlighted } => {
                        newrunstate = RunState::MainMenu {
                            menu_selection: highlighted,
                        }
                    }
                    MainMenuResult::Selected { highlighted } => match highlighted {
                        MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                        MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.ecs);
                            newrunstate = RunState::AwaitingInput;
                            saveload_system::delete_save();
                        }
                        MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu {
                    menu_selection: MainMenuSelection::Quit,
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
                let game_over_result = game_over(ctx);
                match game_over_result {
                    GameOverResult::NoSelection => {}
                    GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        newrunstate = RunState::MainMenu {
                            menu_selection: MainMenuSelection::NewGame,
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
                    newrunstate = RunState::Ticking;
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
                let result = show_cheat_mode(self, ctx);
                match result {
                    CheatMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    CheatMenuResult::NoResponse => {}
                    CheatMenuResult::TeleportToExit => {
                        self.goto_level(1);
                        self.mapgen_next_state = Some(RunState::PreRun);
                        newrunstate = RunState::MapGeneration;
                    }
                    CheatMenuResult::MagicMapper => {
                        // newrunstate = RunState::MagicMapReveal { row: 0 }
                        let mut map = self.ecs.fetch_mut::<Map>();
                        map.revealed_tiles.iter_mut().for_each(|v| *v = true);
                    }
                    CheatMenuResult::Heal => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        player_pools.hit_points.current = player_pools.hit_points.max;
                        newrunstate = RunState::AwaitingInput;
                    }
                    CheatMenuResult::GodMode => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        player_pools.god_mode = true;
                        newrunstate = RunState::AwaitingInput;
                    }
                    CheatMenuResult::GetRich => {
                        let player = self.ecs.fetch::<Entity>();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let player_pools = pools.get_mut(*player).unwrap();
                        player_pools.gold += 100_f32;
                        newrunstate = RunState::AwaitingInput;
                    }
                }
            }
            RunState::ShowVendor { vendor, mode } => {
                let (vendor_result, entity, tag, sell_price) =
                    show_vendor_menu(self, ctx, vendor, mode);
                match vendor_result {
                    VendorResult::Cancel => newrunstate = RunState::AwaitingInput,
                    VendorResult::NoResponse => {}
                    VendorResult::Sell => {
                        let e = entity.unwrap();
                        let price =
                            self.ecs.read_storage::<Item>().get(e).unwrap().base_value * 0.8;
                        // TODO(aalhendi): Clean this up
                        self.ecs
                            .write_storage::<Pools>()
                            .get_mut(*self.ecs.fetch::<Entity>())
                            .unwrap()
                            .gold += price;
                        self.ecs.delete_entity(e).expect("Unable to delete");
                    }
                    VendorResult::Buy => {
                        let tag = tag.unwrap();
                        let price = sell_price.unwrap();
                        let mut pools = self.ecs.write_storage::<Pools>();
                        let mut identified = self.ecs.write_storage::<IdentifiedItem>();
                        let player_entity = self.ecs.fetch::<Entity>();
                        identified
                            .insert(*player_entity, IdentifiedItem { name: tag.clone() })
                            .expect("Unable to insert");
                        std::mem::drop(identified);
                        let player_pools = pools.get_mut(*player_entity).unwrap();
                        std::mem::drop(player_entity);
                        if player_pools.gold >= price {
                            player_pools.gold -= price;
                            std::mem::drop(pools);
                            let player_entity = *self.ecs.fetch::<Entity>();
                            spawn_named_item(
                                &RAWS.lock().unwrap(),
                                &mut self.ecs,
                                &tag,
                                SpawnType::Carried { by: player_entity },
                            );
                        }
                    }
                    VendorResult::BuyMode => newrunstate = RunState::buy_vendor(vendor),
                    VendorResult::SellMode => newrunstate = RunState::sell_vendor(vendor),
                }
            }
            RunState::TownPortal => {
                // Spawn the portal
                spawner::spawn_town_portal(&mut self.ecs);

                // Transition
                let map_depth = self.ecs.fetch::<Map>().depth;
                let destination_offset = 0 - (map_depth - 1);
                self.goto_level(destination_offset);
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::TeleportingToOtherLevel { x, y, depth } => {
                self.goto_level(depth - 1);
                let player_entity = self.ecs.fetch::<Entity>();
                if let Some(pos) = self.ecs.write_storage::<Position>().get_mut(*player_entity) {
                    pos.x = x;
                    pos.y = y;
                }
                let mut ppos = self.ecs.fetch_mut::<rltk::Point>();
                ppos.x = x;
                ppos.y = y;
                self.mapgen_next_state = Some(RunState::PreRun);
                newrunstate = RunState::MapGeneration;
            }
            RunState::ShowRemoveCurse => {
                let (menu_result, maybe_entity) = remove_curse_menu(self, ctx);
                match menu_result {
                    ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
                        let item_entity = maybe_entity.unwrap();
                        self.ecs.write_storage::<CursedItem>().remove(item_entity);
                        newrunstate = RunState::Ticking;
                    }
                }
            }
            RunState::ShowIdentify => {
                let (menu_result, maybe_entity) = identify_menu(self, ctx);
                match menu_result {
                    ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ItemMenuResult::NoResponse => {}
                    ItemMenuResult::Selected => {
                        let item_entity = maybe_entity.unwrap();
                        if let Some(name) = self.ecs.read_storage::<Name>().get(item_entity) {
                            let mut dm = self.ecs.fetch_mut::<MasterDungeonMap>();
                            dm.identified_items.insert(name.name.clone());
                        }
                        newrunstate = RunState::Ticking;
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        damage_system::delete_the_dead(&mut self.ecs);

        if SHOW_FPS {
            let mut draw_batch = rltk::DrawBatch::new();
            draw_batch.print(Point::new(1, 59), format!("FPS: {}", ctx.fps));
            let _ = draw_batch.submit(7000);
        }

        let _ = rltk::render_draw_buffer(ctx);
    }
}
// --- State End ---

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple(80, 60)
        .unwrap()
        .with_title("Rust Roguelike !")
        .with_vsync(false)
        .with_font("vga8x16.png", 8, 16)
        .with_sparse_console(80, 30, "vga8x16.png")
        .build()?;

    let mut gs = State {
        ecs: World::new(),
        mapgen_next_state: Some(RunState::MainMenu {
            menu_selection: MainMenuSelection::NewGame,
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
    gs.ecs.register::<Name>();
    gs.ecs.register::<BlocksTile>();
    gs.ecs.register::<WantsToMelee>();
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
    gs.ecs.register::<Weapon>();
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
    gs.ecs.register::<OtherLevelPosition>();
    gs.ecs.register::<DMSerializationHelper>();
    gs.ecs.register::<LightSource>();
    gs.ecs.register::<Initiative>();
    gs.ecs.register::<MyTurn>();
    gs.ecs.register::<Faction>();
    gs.ecs.register::<WantsToApproach>();
    gs.ecs.register::<WantsToFlee>();
    gs.ecs.register::<MoveMode>();
    gs.ecs.register::<Chasing>();
    gs.ecs.register::<EquipmentChanged>();
    gs.ecs.register::<Vendor>();
    gs.ecs.register::<TownPortal>();
    gs.ecs.register::<TeleportTo>();
    gs.ecs.register::<ApplyMove>();
    gs.ecs.register::<ApplyTeleport>();
    gs.ecs.register::<MagicItem>();
    gs.ecs.register::<ObfuscatedName>();
    gs.ecs.register::<IdentifiedItem>();
    gs.ecs.register::<SpawnParticleBurst>();
    gs.ecs.register::<SpawnParticleLine>();
    gs.ecs.register::<CursedItem>();
    gs.ecs.register::<ProvidesRemoveCurse>();
    gs.ecs.register::<ProvidesIdentification>();
    gs.ecs.register::<AttributeBonus>();
    gs.ecs.register::<Duration>();
    gs.ecs.register::<StatusEffect>();
    gs.ecs.register::<KnownSpells>();
    gs.ecs.register::<WantsToCastSpell>();
    gs.ecs.register::<SpellTemplate>();
    gs.ecs.register::<ProvidesMana>();
    gs.ecs.register::<TeachesSpell>();
    gs.ecs.register::<DamageOverTime>();
    gs.ecs.register::<Slow>();
    gs.ecs.register::<SpecialAbilities>();
    gs.ecs.register::<TileSize>();
    gs.ecs.register::<OnDeath>();
    gs.ecs.register::<AlwaysTargetsSelf>();
    gs.ecs.register::<Target>();
    gs.ecs.register::<WantsToShoot>();

    gs.ecs.insert(SimpleMarkerAllocator::<IsSerialized>::new());
    raws::load_raws();

    // Resource Insertion
    gs.ecs.insert(MasterDungeonMap::new());
    gs.ecs.insert(Map::new(1, 64, 64, "New Map"));
    gs.ecs.insert(Point::new(0, 0));
    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    let player_entity = spawner::player(&mut gs.ecs, Position { x: 0, y: 0 });
    gs.ecs.insert(rex_assets::RexAssets::new());
    gs.ecs.insert(player_entity);
    if SHOW_MAPGEN_VISUALIZER {
        gs.ecs.insert(RunState::MapGeneration {});
    } else {
        gs.ecs.insert(RunState::MainMenu {
            menu_selection: MainMenuSelection::NewGame,
        });
    }

    gamelog::clear_log();
    gamelog::Logger::new()
        .white("Welcome to")
        .cyan("Rusty Roguelike")
        .log();
    gamelog::events::clear_events();

    gs.ecs.insert(particle_system::ParticleBuilder::new());

    gs.generate_world_map(1, 0);

    rltk::main_loop(context, gs)
}
