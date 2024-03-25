use super::{
    BlocksTile, BlocksVisibility, Door, EntityMoved, HungerClock, HungerState, Item, Map, Player,
    Position, Renderable, RunState, State, Viewshed, WantsToPickupItem,
};
use crate::components::WantsToMelee;
use crate::gamelog::Gamelog;
use crate::map::TileType;
use crate::raws::faction_structs::Reaction;
use crate::raws::rawsmaster::faction_reaction;
use crate::raws::rawsmaster::find_spell_entity;
use crate::raws::RAWS;
use crate::spatial;
use crate::Consumable;
use crate::Faction;
use crate::InBackpack;
use crate::KnownSpells;
use crate::Pools;
use crate::Ranged;
use crate::Vendor;
use crate::VendorMode;
use crate::WantsToCastSpell;
use crate::WantsToUseItem;
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
    let mut ppos = ecs.write_resource::<Point>();
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let pools = ecs.read_storage::<Pools>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let mut doors = ecs.write_storage::<Door>();
    let mut blocks_visibility = ecs.write_storage::<BlocksVisibility>();
    let mut blocks_movement = ecs.write_storage::<BlocksTile>();
    let mut renderables = ecs.write_storage::<Renderable>();
    let factions = ecs.read_storage::<Faction>();
    let mut swap_entities = Vec::new();
    let mut result = RunState::AwaitingInput;
    let vendors = ecs.read_storage::<Vendor>();

    for (_player, pos, viewshed, entity) in
        (&mut players, &mut positions, &mut viewsheds, &entities).join()
    {
        // Check bounds
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.height - 1
        {
            return RunState::AwaitingInput; // move wasn't valid
        }
        let dest_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        result = spatial::for_each_tile_content_with_gamemode(dest_idx, |potential_target| {
            // TODO(aalhendi): this returns early and so vendors cannot be hostile
            if vendors.get(potential_target).is_some() {
                return Some(RunState::ShowVendor {
                    vendor: potential_target,
                    mode: VendorMode::Sell,
                });
            }

            let is_hostile = if pools.get(potential_target).is_some() {
                if let Some(faction) = factions.get(potential_target) {
                    let reaction = faction_reaction(&faction.name, "Player", &RAWS.lock().unwrap());
                    reaction == Reaction::Attack
                } else {
                    true
                }
            } else {
                true
            };

            if !is_hostile {
                // Note that we want to move the bystander
                swap_entities.push((potential_target, pos.x, pos.y));

                // Move the player
                pos.x = (pos.x + delta_x).clamp(0, map.width - 1);
                pos.y = (pos.y + delta_y).clamp(0, map.height - 1);
                entity_moved
                    .insert(entity, EntityMoved {})
                    .expect("Unable to insert marker");

                viewshed.dirty = true;
                ppos.x = pos.x;
                ppos.y = pos.y;
                return Some(RunState::Ticking);
            } else if let Some(_tgt) = pools.get(potential_target) {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: potential_target,
                        },
                    )
                    .expect("Add target failed");
                return Some(RunState::Ticking); // So we don't move after attacking
            }

            if let Some(door) = doors.get_mut(potential_target) {
                door.open = true;
                blocks_visibility.remove(potential_target);
                blocks_movement.remove(potential_target);
                let door_renderable = renderables.get_mut(potential_target).unwrap();
                door_renderable.glyph = rltk::to_cp437('/');
                viewshed.dirty = true;
                return Some(RunState::Ticking);
            }
            None
        });

        if !spatial::is_blocked(dest_idx) {
            pos.x = (pos.x + delta_x).clamp(0, map.width - 1);
            pos.y = (pos.y + delta_y).clamp(0, map.height - 1);
            entity_moved
                .insert(entity, EntityMoved {})
                .expect("Unable to insert marker");

            viewshed.dirty = true;
            ppos.x = pos.x;
            ppos.y = pos.y;

            result = match map.tiles[dest_idx] {
                TileType::DownStairs => RunState::NextLevel,
                TileType::UpStairs => RunState::PreviousLevel,
                _ => RunState::Ticking,
            };
        }
    }

    for (swappable_entity, swappable_pos_x, swappable_pos_y) in swap_entities {
        if let Some(e_pos) = positions.get_mut(swappable_entity) {
            let old_idx = map.xy_idx(e_pos.x, e_pos.y);
            let new_idx = map.xy_idx(swappable_pos_x, swappable_pos_y);
            e_pos.x = swappable_pos_x;
            e_pos.y = swappable_pos_y;
            spatial::move_entity(swappable_entity, old_idx, new_idx);
            result = RunState::Ticking;
        }
    }
    result
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    if let Some(key) = ctx.key {
        use VirtualKeyCode::*;
        // Hotkeys (Shift held down)
        if ctx.shift {
            let key_val = match key {
                Key1 => Some(1),
                Key2 => Some(2),
                Key3 => Some(3),
                Key4 => Some(4),
                Key5 => Some(5),
                Key6 => Some(6),
                Key7 => Some(7),
                Key8 => Some(8),
                Key9 => Some(9),
                _ => None,
            };
            if let Some(key_val) = key_val {
                return use_consumable_hotkey(gs, key_val - 1);
            }
        }

        if ctx.control {
            let key_val = match key {
                Key1 => Some(1),
                Key2 => Some(2),
                Key3 => Some(3),
                Key4 => Some(4),
                Key5 => Some(5),
                Key6 => Some(6),
                Key7 => Some(7),
                Key8 => Some(8),
                Key9 => Some(9),
                _ => None,
            };
            if let Some(key_val) = key_val {
                return use_spell_hotkey(gs, key_val - 1);
            }
        }

        match key {
            // Skip turn
            Space | Numpad5 => skip_turn(&mut gs.ecs),

            // Cardinal
            Left | Numpad4 | H => try_move_player(-1, 0, &mut gs.ecs),
            Right | Numpad6 | L => try_move_player(1, 0, &mut gs.ecs),
            Up | Numpad8 | K => try_move_player(0, -1, &mut gs.ecs),
            Down | Numpad2 | J => try_move_player(0, 1, &mut gs.ecs),

            //Diagonal
            Numpad7 | Y => try_move_player(-1, -1, &mut gs.ecs),
            Numpad3 | N => try_move_player(1, 1, &mut gs.ecs),
            Numpad1 | B => try_move_player(-1, 1, &mut gs.ecs),
            Numpad9 | U => try_move_player(1, -1, &mut gs.ecs),

            // Item
            G => {
                get_item(&mut gs.ecs);
                RunState::Ticking
            }
            I => RunState::ShowInventory,
            D => RunState::ShowDropItem,
            R => RunState::ShowRemoveItem,

            // Main Menu
            Escape => RunState::SaveGame,
            // Cheating!
            Backslash => RunState::ShowCheatMenu,
            // Stairs
            Period => {
                if try_next_level(&mut gs.ecs) {
                    RunState::NextLevel
                } else {
                    RunState::Ticking
                }
            }
            Comma => {
                if try_previous_level(&mut gs.ecs) {
                    RunState::PreviousLevel
                } else {
                    RunState::Ticking
                }
            }
            _ => RunState::AwaitingInput,
        }
    } else {
        RunState::AwaitingInput
    }
}

fn use_consumable_hotkey(gs: &mut State, key: usize) -> RunState {
    let consumables = gs.ecs.read_storage::<Consumable>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let player_entity = gs.ecs.fetch::<Entity>();
    let entities = gs.ecs.entities();

    let mut carried_consumables = Vec::new();
    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumables).join() {
        if carried_by.owner == *player_entity {
            carried_consumables.push(entity);
        }
    }

    if key < carried_consumables.len() {
        if let Some(ranged) = gs
            .ecs
            .read_storage::<Ranged>()
            .get(carried_consumables[key])
        {
            return RunState::ShowTargeting {
                range: ranged.range,
                item: carried_consumables[key],
            };
        }
        let mut intent = gs.ecs.write_storage::<WantsToUseItem>();
        intent
            .insert(
                *player_entity,
                WantsToUseItem {
                    item: carried_consumables[key],
                    target: None,
                },
            )
            .expect("Unable to insert intent");
        return RunState::Ticking;
    }
    RunState::Ticking
}

fn use_spell_hotkey(gs: &mut State, key: usize) -> RunState {
    let player_entity = gs.ecs.fetch::<Entity>();
    let known_spells_storage = gs.ecs.read_storage::<KnownSpells>();
    let known_spells = &known_spells_storage.get(*player_entity).unwrap().spells;

    if key < known_spells.len() {
        let pools = gs.ecs.read_storage::<Pools>();
        let player_pools = pools.get(*player_entity).unwrap();
        if player_pools.mana.current >= known_spells[key].mana_cost {
            if let Some(spell_entity) = find_spell_entity(&gs.ecs, &known_spells[key].display_name)
            {
                if let Some(ranged) = gs.ecs.read_storage::<Ranged>().get(spell_entity) {
                    return RunState::ShowTargeting {
                        range: ranged.range,
                        item: spell_entity,
                    };
                };
                let mut intent = gs.ecs.write_storage::<WantsToCastSpell>();
                intent
                    .insert(
                        *player_entity,
                        WantsToCastSpell {
                            spell: spell_entity,
                            target: None,
                        },
                    )
                    .expect("Unable to insert intent");
                return RunState::Ticking;
            }
        } else {
            let mut gamelog = gs.ecs.fetch_mut::<Gamelog>();
            gamelog
                .entries
                .push("You don't have enough mana to cast that!".to_string());
        }
    }

    RunState::Ticking
}

fn get_item(ecs: &mut World) {
    // TODO: Can't we grab pos from player entity?
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<Gamelog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        // TODO: Positon-Point equality
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(
                    *player_entity,
                    WantsToPickupItem {
                        collected_by: *player_entity,
                        item,
                    },
                )
                .expect("Unable to insert want to pickup");
        }
    }
}

fn try_next_level(ecs: &mut World) -> bool {
    let p_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(p_pos.x, p_pos.y);
    let is_down_stairs = map.tiles[player_idx] == TileType::DownStairs;
    if !is_down_stairs {
        let mut gamelog = ecs.fetch_mut::<Gamelog>();
        gamelog
            .entries
            .push("There is no way down from here".to_string());
    }
    is_down_stairs
}

fn try_previous_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    let is_up_stairs = map.tiles[player_idx] == TileType::UpStairs;
    if !is_up_stairs {
        let mut gamelog = ecs.fetch_mut::<Gamelog>();
        gamelog
            .entries
            .push("There is no way up from here".to_string());
    }
    is_up_stairs
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let factions = ecs.read_storage::<Faction>();
    let map = ecs.fetch::<Map>();
    let mut can_heal = true;

    // Check that no monsters in player viewshed
    let viewshed = viewsheds.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = map.xy_idx(tile.x, tile.y);
        spatial::for_each_tile_content(idx, |entity| {
            if let Some(f) = factions.get(entity) {
                let reaction = faction_reaction(&f.name, "Player", &RAWS.lock().unwrap());
                if reaction == Reaction::Attack {
                    can_heal = false;
                }
            }
        });
    }

    let hunger_clocks = ecs.read_storage::<HungerClock>();
    if let Some(hc) = hunger_clocks.get(*player_entity) {
        match hc.state {
            HungerState::Hungry => can_heal = false,
            HungerState::Starving => can_heal = false,
            _ => {}
        }
    }

    // Heal Player
    if can_heal {
        let mut pools = ecs.write_storage::<Pools>();
        let player_stats = pools.get_mut(*player_entity).unwrap();
        player_stats.hit_points.current = i32::min(
            player_stats.hit_points.current + 1,
            player_stats.hit_points.max,
        );
        let mut rng = ecs.fetch_mut::<rltk::RandomNumberGenerator>();
        if rng.roll_dice(1, 6) == 1 {
            player_stats.mana.current =
                i32::min(player_stats.mana.current + 1, player_stats.mana.max);
        }
    }
    RunState::Ticking
}
