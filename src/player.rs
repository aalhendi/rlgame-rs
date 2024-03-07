use super::{
    BlocksTile, BlocksVisibility, Door, EntityMoved, HungerClock, HungerState, Item, Map, Monster,
    Player, Position, Renderable, RunState, State, Viewshed, WantsToPickupItem,
};
use crate::components::Bystander;
use crate::components::Vendor;
use crate::components::WantsToMelee;
use crate::gamelog::Gamelog;
use crate::map::TileType;
use crate::Consumable;
use crate::InBackpack;
use crate::Pools;
use crate::Ranged;
use crate::WantsToUseItem;
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
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
    let bystanders = ecs.read_storage::<Bystander>();
    let vendors = ecs.read_storage::<Vendor>();
    let mut swap_entities = Vec::new();

    for (_player, pos, viewshed, entity) in
        (&mut players, &mut positions, &mut viewsheds, &entities).join()
    {
        // Check bounds
        if pos.x + delta_x < 1
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 1
            || pos.y + delta_y > map.height - 1
        {
            return;
        }
        let dest_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[dest_idx].iter() {
            if bystanders.get(*potential_target).is_some()
                || vendors.get(*potential_target).is_some()
            {
                swap_entities.push((*potential_target, *pos));
                pos.x = (pos.x + delta_x).clamp(0, map.width - 1);
                pos.y = (pos.y + delta_y).clamp(0, map.height - 1);
                entity_moved
                    .insert(entity, EntityMoved {})
                    .expect("Unable to insert marker");

                viewshed.dirty = true;
                ppos.x = pos.x;
                ppos.y = pos.y;
            } else if pools.get(*potential_target).is_some() {
                wants_to_melee
                    .insert(
                        entity,
                        WantsToMelee {
                            target: *potential_target,
                        },
                    )
                    .expect("Add target failed");
                return; // So we don't move after attacking
            }

            if let Some(door) = doors.get_mut(*potential_target) {
                door.open = true;
                blocks_visibility.remove(*potential_target);
                blocks_movement.remove(*potential_target);
                let door_renderable = renderables.get_mut(*potential_target).unwrap();
                door_renderable.glyph = rltk::to_cp437('/');
                viewshed.dirty = true;
            }
        }

        if !map.blocked[dest_idx] {
            pos.x = (pos.x + delta_x).clamp(0, map.width - 1);
            pos.y = (pos.y + delta_y).clamp(0, map.height - 1);
            ppos.x = pos.x;
            ppos.y = pos.y;

            viewshed.dirty = true;
        }
    }

    for (swappable_entity, swappable_pos) in swap_entities.iter() {
        if let Some(e_pos) = positions.get_mut(*swappable_entity) {
            e_pos.x = swappable_pos.x;
            e_pos.y = swappable_pos.y;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    use VirtualKeyCode::*;
    // TODO: Replace with if let
    match ctx.key {
        None => return RunState::AwaitingInput,
        // Hotkeys (Shift held down)
        Some(key) if ctx.shift => {
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
        Some(key) => match key {
            // Skip turn
            Space | Numpad5 => return skip_turn(&mut gs.ecs),

            // Cardinal
            Left | Numpad4 | H => try_move_player(-1, 0, &mut gs.ecs),
            Right | Numpad6 | L => try_move_player(1, 0, &mut gs.ecs),
            Up | Numpad8 | K => try_move_player(0, -1, &mut gs.ecs),
            Down | Numpad2 | J => try_move_player(0, 1, &mut gs.ecs),

            //Diagonal
            Numpad1 | Y => try_move_player(-1, -1, &mut gs.ecs),
            Numpad9 | N => try_move_player(1, 1, &mut gs.ecs),
            Numpad7 | B => try_move_player(-1, 1, &mut gs.ecs),
            Numpad3 | U => try_move_player(1, -1, &mut gs.ecs),

            // Item
            G => get_item(&mut gs.ecs),
            I => return RunState::ShowInventory,
            D => return RunState::ShowDropItem,
            R => return RunState::ShowRemoveItem,

            // Main Menu
            Escape => return RunState::SaveGame,

            // Stairs
            Period => {
                if is_down_stairs(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            }

            _ => return RunState::AwaitingInput,
        },
    }
    RunState::PlayerTurn
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
        return RunState::PlayerTurn;
    }
    RunState::PlayerTurn
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

pub fn is_down_stairs(ecs: &mut World) -> bool {
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

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewsheds = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();
    let map = ecs.fetch::<Map>();

    // Check that no monsters in player viewshed
    let viewshed = viewsheds.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = map.xy_idx(tile.x, tile.y);
        for entity in map.tile_content[idx].iter() {
            if monsters.get(*entity).is_some() {
                return RunState::PlayerTurn;
            }
        }
    }

    let hunger_clocks = ecs.read_storage::<HungerClock>();
    if let Some(hc) = hunger_clocks.get(*player_entity) {
        match hc.state {
            HungerState::Hungry => return RunState::PlayerTurn,
            HungerState::Starving => return RunState::PlayerTurn,
            _ => {}
        }
    }

    // Heal Player
    let mut pools = ecs.write_storage::<Pools>();
    let player_stats = pools.get_mut(*player_entity).unwrap();
    player_stats.hit_points.current = i32::min(
        player_stats.hit_points.current + 1,
        player_stats.hit_points.max,
    );
    RunState::PlayerTurn
}
