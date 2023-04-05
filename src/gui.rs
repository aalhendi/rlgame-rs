use crate::camera;

use super::{
    gamelog::Gamelog, CombatStats, Equipped, Hidden, HungerClock, HungerState, InBackpack, Map,
    Name, Owned, Player, Position, RunState, State, Viewshed,
};
use rltk::{Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(
        0,
        43,
        79,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    // TODO: If player is a resource in the ECS, can't we just fetch it insead of
    // player entity and combat_stats component read calls?
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let log = ecs.fetch::<Gamelog>();
    let hunger = ecs.read_storage::<HungerClock>();

    let mut y = 44;
    for entry in log.entries.iter().rev() {
        if y < 49 {
            ctx.print(2, y, entry);
        }
        y += 1;
    }

    let yellow = RGB::named(rltk::YELLOW);
    let black = RGB::named(rltk::BLACK);
    let red = RGB::named(rltk::RED);
    let magenta = RGB::named(rltk::MAGENTA);
    let green = RGB::named(rltk::GREEN);
    let orange = RGB::named(rltk::ORANGE);
    let white = RGB::named(rltk::WHITE);

    for (_player, stats, hc) in (&players, &combat_stats, &hunger).join() {
        let health = format!(
            " HP: {hp} / {max_hp} ",
            hp = stats.hp,
            max_hp = stats.max_hp
        );
        ctx.print_color(12, 43, yellow, black, &health);

        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, red, black);

        match hc.state {
            HungerState::WellFed => ctx.print_color(71, 42, green, black, "Well Fed"),
            HungerState::Normal => ctx.print_color(71, 42, white, black, "Normal"),
            HungerState::Hungry => ctx.print_color(71, 42, orange, black, "Hungry"),
            HungerState::Starving => ctx.print_color(71, 42, red, black, "Starving"),
        }
    }

    let map = ecs.fetch::<Map>();
    let depth = format!("Depth: {depth}", depth = map.depth);
    ctx.print_color(2, 43, yellow, black, &depth);

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, magenta);
    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    let white = RGB::named(rltk::WHITE);
    let grey = RGB::named(rltk::GREY);

    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();
    let hidden = ecs.read_storage::<Hidden>();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;

    // Check if mouse is on map
    if mouse_map_pos.0 >= map.width - 1
        || mouse_map_pos.1 >= map.height - 1
        || mouse_map_pos.0 < 1
        || mouse_map_pos.1 < 1
    {
        return;
    }
    if !map.visible_tiles[map.xy_idx(mouse_map_pos.0, mouse_map_pos.1)] {
        return;
    }

    let mut tooltip: Vec<String> = Vec::new();

    for (name, pos, _hidden) in (&names, &positions, !&hidden).join() {
        if pos.x == mouse_map_pos.0 && pos.y == mouse_map_pos.1 {
            tooltip.push(name.name.to_string());
        }
    }

    if !tooltip.is_empty() {
        let mut width = 0;
        for s in tooltip.iter() {
            if width < s.len() {
                width = s.len();
            }
        }
        width += 3;

        if mouse_pos.0 > map.width / 2 {
            // Left label
            let arrow_pos = Point::new(mouse_pos.0 - 2, mouse_pos.1);
            let left_x = mouse_pos.0 - width as i32;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x, y, white, grey, s);
                let padding = (width - s.len()) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x - i as i32, y, white, grey, &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, white, grey, &"->".to_string());
        } else {
            // Right label
            let arrow_pos = Point::new(mouse_pos.0 + 1, mouse_pos.1);
            let left_x = mouse_pos.0 + 3;
            let mut y = mouse_pos.1;
            for s in tooltip.iter() {
                ctx.print_color(left_x + 1, y, white, grey, s);
                let padding = (width - s.len()) - 1;
                for i in 0..padding {
                    ctx.print_color(arrow_pos.x + 1 + i as i32, y, white, grey, &" ".to_string());
                }
                y += 1;
            }
            ctx.print_color(arrow_pos.x, arrow_pos.y, white, grey, &"<-".to_string());
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
}

pub fn show_menu<T: Owned + Component>(
    gs: &mut State,
    ctx: &mut Rltk,
) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<T>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|(item, _name)| item.owned_by(&player_entity));
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    print_item_menu(ctx, y, count, "Inventory");

    let mut equippable: Vec<Entity> = Vec::new();
    for (j, (entity, _pack, item_name)) in (&entities, &backpack, &names)
        .join()
        .filter(|(_entity, item, _name)| item.owned_by(&player_entity))
        .enumerate()
    {
        let label_char = char::from_u32((97 + j) as u32).expect("Invalid char");
        print_item_label(ctx, y, label_char, item_name);
        equippable.push(entity);
        y += 1;
    }

    item_menu_input(ctx.key, &equippable, count as i32)
}

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    show_menu::<InBackpack>(gs, ctx)
}

pub fn drop_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    show_menu::<InBackpack>(gs, ctx)
}

pub fn remove_item_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    show_menu::<Equipped>(gs, ctx)
}

fn print_item_label(ctx: &mut Rltk, y: i32, label_char: char, name: &Name) {
    ctx.set(
        17,
        y,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437('('),
    );
    ctx.set(
        18,
        y,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        rltk::to_cp437(label_char),
    );
    ctx.set(
        19,
        y,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        rltk::to_cp437(')'),
    );

    ctx.print(21, y, &name.name.to_string());
}

fn item_menu_input(
    key: Option<VirtualKeyCode>,
    items: &[Entity],
    count: i32,
) -> (ItemMenuResult, Option<Entity>) {
    match key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count {
                    return (ItemMenuResult::Selected, Some(items[selection as usize]));
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
}

pub fn ranged_target(
    gs: &mut State,
    ctx: &mut Rltk,
    range: i32,
) -> (ItemMenuResult, Option<Point>) {
    let (min_x, max_x, min_y, max_y) = camera::get_screen_bounds(&gs.ecs, ctx);
    let player_entity = gs.ecs.fetch::<Entity>();
    let player_pos = gs.ecs.fetch::<Point>();
    let viewsheds = gs.ecs.read_storage::<Viewshed>();

    ctx.print_color(
        5,
        0,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Select Target:",
    );

    // Highlight available target cells
    let mut available_cells = Vec::new();
    if let Some(p_viewshed) = viewsheds.get(*player_entity) {
        // We have a viewshed
        for pt in p_viewshed.visible_tiles.iter() {
            let distance = rltk::DistanceAlg::Pythagoras.distance2d(*player_pos, *pt);
            if distance <= range as f32 {
                let screen_x = pt.x - min_x;
                let screen_y = pt.y - min_y;
                if screen_x > 1
                    && screen_x < (max_x - min_x) - 1
                    && screen_y > 1
                    && screen_y < (max_y - min_y) - 1
                {
                    ctx.set_bg(screen_x, screen_y, RGB::named(rltk::BLUE));
                    available_cells.push(pt);
                }
            }
        }
    } else {
        return (ItemMenuResult::Cancel, None);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    mouse_map_pos.0 += min_x;
    mouse_map_pos.1 += min_y;

    let mut valid_target = false;
    for idx in available_cells.iter() {
        if idx.x == mouse_map_pos.0 && idx.y == mouse_map_pos.1 {
            valid_target = true;
        }
    }

    if valid_target {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::CYAN));
        if ctx.left_click {
            return (
                ItemMenuResult::Selected,
                Some(Point::new(mouse_map_pos.0, mouse_map_pos.1)),
            );
        }
    } else {
        ctx.set_bg(mouse_pos.0, mouse_pos.1, RGB::named(rltk::RED));
        if ctx.left_click {
            return (ItemMenuResult::Cancel, None);
        }
    }

    (ItemMenuResult::NoResponse, None)
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection {
    NewGame,
    LoadGame,
    Quit,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuResult {
    NoSelection { highlighted: MainMenuSelection },
    Selected { highlighted: MainMenuSelection },
}

pub fn main_menu(gs: &mut State, ctx: &mut Rltk) -> MainMenuResult {
    let save_exists = super::saveload_system::save_exists();
    let runstate = gs.ecs.fetch::<RunState>();

    ctx.print_color_centered(
        15,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Rust Roguelike Tutorial",
    );

    if let RunState::MainMenu {
        menu_selection: cur_hovering,
    } = *runstate
    {
        print_menu_item(
            ctx,
            "Begin New Game",
            24,
            cur_hovering == MainMenuSelection::NewGame,
        );

        if save_exists {
            print_menu_item(
                ctx,
                "Load Game",
                25,
                cur_hovering == MainMenuSelection::LoadGame,
            );
        }
        print_menu_item(ctx, "Quit", 26, cur_hovering == MainMenuSelection::Quit);

        if let Some(key) = ctx.key {
            match key {
                VirtualKeyCode::Escape => {
                    return MainMenuResult::NoSelection {
                        highlighted: MainMenuSelection::Quit,
                    }
                }
                VirtualKeyCode::Up => {
                    // Cycle++
                    return MainMenuResult::NoSelection {
                        highlighted: cycle_hovering(cur_hovering, true, save_exists),
                    };
                }
                VirtualKeyCode::Down => {
                    //Cycle--
                    return MainMenuResult::NoSelection {
                        highlighted: cycle_hovering(cur_hovering, false, save_exists),
                    };
                }
                VirtualKeyCode::Return => {
                    return MainMenuResult::Selected {
                        highlighted: cur_hovering,
                    }
                }
                _ => {
                    return MainMenuResult::NoSelection {
                        highlighted: cur_hovering,
                    }
                }
            }
        } else {
            return MainMenuResult::NoSelection {
                highlighted: cur_hovering,
            };
        }
    }

    MainMenuResult::NoSelection {
        highlighted: MainMenuSelection::NewGame,
    }
}

fn print_menu_item(ctx: &mut Rltk, text: &str, y: i32, is_highlighted: bool) {
    let fg = {
        if is_highlighted {
            RGB::named(rltk::MAGENTA)
        } else {
            RGB::named(rltk::WHITE)
        }
    };
    ctx.print_color_centered(y, fg, RGB::named(rltk::BLACK), text);
}

fn cycle_hovering(
    cur_hovering: MainMenuSelection,
    is_positive_direction: bool,
    save_exists: bool,
) -> MainMenuSelection {
    if is_positive_direction {
        match cur_hovering {
            MainMenuSelection::NewGame => MainMenuSelection::Quit,
            MainMenuSelection::LoadGame => MainMenuSelection::NewGame,
            MainMenuSelection::Quit => {
                if save_exists {
                    MainMenuSelection::LoadGame
                } else {
                    MainMenuSelection::NewGame
                }
            }
        }
    } else {
        match cur_hovering {
            MainMenuSelection::NewGame => {
                if save_exists {
                    MainMenuSelection::LoadGame
                } else {
                    MainMenuSelection::Quit
                }
            }
            MainMenuSelection::LoadGame => MainMenuSelection::Quit,
            MainMenuSelection::Quit => MainMenuSelection::NewGame,
        }
    }
}

fn print_item_menu(ctx: &mut Rltk, y: i32, count: usize, label: &str) {
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        label,
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );
}

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    ctx.print_color_centered(
        15,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "You Died!",
    );
    ctx.print_color_centered(
        18,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
        "Some day there might be stats here...",
    );

    ctx.print_color_centered(
        20,
        RGB::named(rltk::MAGENTA),
        RGB::named(rltk::BLACK),
        "Press any key to return to the menu.",
    );

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu,
    }
}
