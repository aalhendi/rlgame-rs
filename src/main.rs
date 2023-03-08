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
mod hunger_system;
mod particle_system;
mod random_table;
mod saveload_system;
mod trigger_system;

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
}

pub struct State {
    pub ecs: World,
}

impl State {
    fn goto_next_level(&mut self) {
        let to_delete = self.entities_to_remove_on_level_change();
        for e in to_delete {
            self.ecs.delete_entity(e).expect("Unable to delte entity");
        }

        let worldmap = {
            let mut wmap_res = self.ecs.write_resource::<Map>();
            *wmap_res = Map::new_map_rooms_and_corridors(wmap_res.depth + 1);
            wmap_res.clone()
        };

        // Populate rooms
        for room in worldmap.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room, worldmap.depth);
        }

        let p_pos = worldmap.rooms[0].center();
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(p_pos.x, p_pos.y);

        let mut positions_store = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        if let Some(ppos_comp) = positions_store.get_mut(*player_entity) {
            *ppos_comp = p_pos;
        }

        let mut viewsheds_store = self.ecs.write_storage::<Viewshed>();
        if let Some(player_vs) = viewsheds_store.get_mut(*player_entity) {
            player_vs.dirty = true;
        }

        // Notify the player and give them some health
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

        // Build a new map and place the player
        let worldmap = {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            *worldmap_resource = Map::new_map_rooms_and_corridors(1);
            worldmap_resource.clone()
        };

        // Spawn bad guys
        for room in worldmap.rooms.iter().skip(1) {
            spawner::spawn_room(&mut self.ecs, room, 1);
        }

        // Place the player and update resources
        let player_pos = worldmap.rooms[0].center();
        let player_entity = spawner::player(&mut self.ecs, player_pos.clone());
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_pos.x, player_pos.y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let mut player_entity_writer = self.ecs.write_resource::<Entity>();
        *player_entity_writer = player_entity;
        if let Some(player_pos_comp) = position_components.get_mut(player_entity) {
            player_pos_comp.x = player_pos.x;
            player_pos_comp.y = player_pos.y;
        }

        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        if let Some(player_vs) = viewshed_components.get_mut(player_entity) {
            player_vs.dirty = true;
        }
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
                draw_map(&self.ecs, ctx);
                {
                    let map = self.ecs.fetch::<Map>();
                    let positions = self.ecs.read_storage::<Position>();
                    let renderables = self.ecs.read_storage::<Renderable>();
                    let hidden = self.ecs.read_storage::<Hidden>();

                    let mut data = (&positions, &renderables, !&hidden)
                        .join()
                        .collect::<Vec<_>>();
                    data.sort_by(
                        |&(_a_pos, a_rndr, _a_hidden), &(_b_pos, b_rndr, _b_hidden)| {
                            b_rndr.render_order.cmp(&a_rndr.render_order)
                        },
                    );
                    for (pos, render, _hidden) in data.iter() {
                        let idx = map.xy_idx(pos.x, pos.y);
                        if map.visible_tiles[idx] {
                            ctx.set(pos.x, pos.y, render.fg, render.bg, render.glyph)
                        }
                    }

                    gui::draw_ui(&self.ecs, ctx);
                }
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

    let mut gs = State { ecs: World::new() };

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

    gs.ecs.insert(SimpleMarkerAllocator::<IsSerialized>::new());

    let map = Map::new_map_rooms_and_corridors(1);
    let player_pos = map.rooms[0].center();

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    for room in map.rooms.iter().skip(1) {
        spawner::spawn_room(&mut gs.ecs, room, 1);
    }

    let player_entity = spawner::player(&mut gs.ecs, player_pos.clone());

    // Resource Insertion
    gs.ecs.insert(RunState::MainMenu {
        menu_selection: gui::MainMenuSelection::NewGame,
    });
    gs.ecs.insert(Point::new(player_pos.x, player_pos.y));
    gs.ecs.insert(map);
    gs.ecs.insert(player_entity);
    gs.ecs.insert(gamelog::Gamelog {
        entries: vec!["Welcome to Rusty Rougelike".to_string()],
    });
    gs.ecs.insert(particle_system::ParticleBuilder::new());

    rltk::main_loop(context, gs)
}
