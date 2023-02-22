use super::{Item, Map, Player, Position, RunState, State, Viewshed, WantsToPickupItem};
use crate::components::CombatStats;
use crate::components::WantsToMelee;
use crate::gamelog::Gamelog;
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut ppos = ecs.write_resource::<Point>();
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let entities = ecs.entities();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();

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
            let target = combat_stats.get(*potential_target);
            if target.is_some() {
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
        }

        if !map.blocked[dest_idx] {
            pos.x = (pos.x + delta_x).clamp(0, map.width - 1);
            pos.y = (pos.y + delta_y).clamp(0, map.height - 1);
            ppos.x = pos.x;
            ppos.y = pos.y;

            viewshed.dirty = true;
        }
    }
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    use VirtualKeyCode::*;
    // TODO: Replace with if let
    match ctx.key {
        None => return RunState::AwaitingInput,
        Some(key) => match key {
            //Cardinal
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

            _ => return RunState::AwaitingInput,
        },
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
