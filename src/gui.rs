use crate::{
    camera::{self, PANE_WIDTH},
    dungeon::MasterDungeonMap,
    raws::{rawsmaster::get_vendor_items, RAWS},
    spatial, Attribute, Attributes, Consumable, CursedItem, Duration, Item, KnownSpells, MagicItem,
    MagicItemClass, ObfuscatedName, Pools, StatusEffect, Vendor, VendorMode,
};

use super::{
    gamelog::Gamelog, Equipped, Hidden, HungerClock, HungerState, InBackpack, Map, Name, Owned,
    RunState, State, Viewshed,
};
use rltk::{Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    use rltk::to_cp437;

    let box_gray: RGB = RGB::from_hex("#999999").expect("Failed to convert hex to RGB.");
    let black = RGB::named(rltk::BLACK);
    let white = RGB::named(rltk::WHITE);
    let red = RGB::named(rltk::RED);
    let blue = RGB::named(rltk::BLUE);
    let cyan = RGB::named(rltk::CYAN);
    let green = RGB::named(rltk::GREEN);
    let yellow = RGB::named(rltk::YELLOW);
    let orange = RGB::named(rltk::ORANGE);
    let magenta = RGB::named(rltk::MAGENTA);
    let gold = RGB::named(rltk::GOLD);

    // Overall box
    draw_hollow_box(ctx, 0, 0, 79, 59, box_gray, black);
    // Map box
    draw_hollow_box(ctx, 0, 0, 49, 45, box_gray, black);
    // Log box
    draw_hollow_box(ctx, 0, 45, 79, 14, box_gray, black);
    // Top-right (Attributes) panel
    draw_hollow_box(ctx, 49, 0, 30, 8, box_gray, black);

    // Box connectors for style
    ctx.set(0, 45, box_gray, black, to_cp437('├'));
    ctx.set(49, 8, box_gray, black, to_cp437('├'));
    ctx.set(49, 0, box_gray, black, to_cp437('┬'));
    ctx.set(49, 45, box_gray, black, to_cp437('┴'));
    ctx.set(79, 8, box_gray, black, to_cp437('┤'));
    ctx.set(79, 45, box_gray, black, to_cp437('┤'));

    // Town Name
    let map = ecs.fetch::<Map>();
    let name_length = (map.name.len() + 2) as i32; // +2 for wrapping char
    let x_pos = PANE_WIDTH - (name_length / 2);
    // Endcap / wrapping chars
    ctx.set(x_pos, 0, box_gray, black, to_cp437('┤'));
    ctx.set(x_pos + name_length - 1, 0, box_gray, black, to_cp437('├'));
    ctx.print_color(x_pos + 1, 0, white, black, &map.name);
    std::mem::drop(map);

    // Stats
    let player_entity = ecs.fetch::<Entity>();
    let pools = ecs.read_storage::<Pools>();
    let player_pools = pools.get(*player_entity).unwrap();
    let entities = ecs.entities();

    let health_curr = player_pools.hit_points.current;
    let health_max = player_pools.hit_points.max;
    let health = format!("Health: {health_curr}/{health_max}",);

    let mana_curr = player_pools.mana.current;
    let mana_max = player_pools.mana.max;
    let mana = format!("Mana: {mana_curr}/{mana_max}",);

    let xp_level_start = (player_pools.level - 1) * 1000;
    let xp_level_curr = player_pools.xp - xp_level_start;
    let level = format!("Level: {lvl}", lvl = player_pools.level);

    ctx.print_color(50, 1, white, black, health);
    ctx.print_color(50, 2, white, black, mana);
    ctx.print_color(50, 3, white, black, level);
    ctx.draw_bar_horizontal(64, 1, 14, health_curr, health_max, red, black);
    ctx.draw_bar_horizontal(64, 2, 14, mana_curr, mana_max, blue, black);
    ctx.draw_bar_horizontal(64, 3, 14, xp_level_curr, 1000, gold, black);

    // Attributes
    let attributes = ecs.read_storage::<Attributes>();
    let p_attr = attributes.get(*player_entity).unwrap();
    draw_attribute("Might:", &p_attr.might, 4, ctx);
    draw_attribute("Quickness:", &p_attr.quickness, 5, ctx);
    draw_attribute("Fitness:", &p_attr.fitness, 6, ctx);
    draw_attribute("Intelligence:", &p_attr.intelligence, 7, ctx);

    // Initiative and Weight
    let weight_str = &format!(
        "{weight:.0} lbs ({weight_max} lbs max)",
        weight = player_pools.total_weight,
        weight_max = (p_attr.might.base + p_attr.might.modifiers) * 15
    );
    let initiative_str = &format!(
        "Initiative Penalty: {penalty:.0}",
        penalty = player_pools.total_initiative_penalty
    );
    ctx.print_color(50, 9, white, black, weight_str);
    ctx.print_color(50, 10, white, black, initiative_str);

    // Gold
    let gold_str = &format!("Gold: {amt:.1}", amt = player_pools.gold);
    ctx.print_color(50, 11, gold, black, gold_str);

    // Wearables / Equipped
    let mut y = 13; // Starting pt
    let equipped = ecs.read_storage::<Equipped>();
    for (entity, equipped_by) in (&entities, &equipped).join() {
        if equipped_by.owner == *player_entity {
            let name = &get_item_display_name(ecs, entity);
            ctx.print_color(50, y, get_item_color(ecs, entity), black, name);
            y += 1;
        }
    }

    // Consumables
    y += 1;
    let consumables = ecs.read_storage::<Consumable>();
    let backpack = ecs.read_storage::<InBackpack>();
    let mut index = 1;
    for (entity, carried_by, _consumable) in (&entities, &backpack, &consumables).join() {
        if carried_by.owner == *player_entity && index < 10 {
            let name = &get_item_display_name(ecs, entity);
            ctx.print_color(50, y, yellow, black, &format!("↑{index}"));
            ctx.print_color(53, y, get_item_color(ecs, entity), black, name);
            y += 1;
            index += 1;
        }
    }

    // Spells
    y += 1;
    let known_spells_storage = ecs.read_storage::<KnownSpells>();
    let known_spells = &known_spells_storage.get(*player_entity).unwrap().spells;
    for (idx, spell) in known_spells.iter().enumerate() {
        ctx.print_color(50, y, cyan, black, &format!("^{idx}", idx = idx + 1));
        let spell_str = &format!("{} ({})", spell.display_name, spell.mana_cost);
        ctx.print_color(53, y, cyan, black, spell_str);
        y += 1;
    }

    // Status
    let mut y = PANE_WIDTH;
    let statuses = ecs.read_storage::<StatusEffect>();
    let durations = ecs.read_storage::<Duration>();
    let names = ecs.read_storage::<Name>();
    let hunger = ecs.read_storage::<HungerClock>();
    let hc = hunger.get(*player_entity).unwrap();
    match hc.state {
        HungerState::WellFed => ctx.print_color(50, y, green, black, "Well Fed"),
        HungerState::Normal => ctx.print_color(50, y, white, black, "Normal"),
        HungerState::Hungry => ctx.print_color(50, y, orange, black, "Hungry"),
        HungerState::Starving => ctx.print_color(50, y, red, black, "Starving"),
    }
    if !matches!(hc.state, HungerState::Normal) {
        y -= 1;
    }

    for (status, duration, name) in (&statuses, &durations, &names).join() {
        if status.target != *player_entity {
            continue;
        }

        let duration_str = &format!("{} ({})", name.name, duration.turns);
        ctx.print_color(50, y, red, black, duration_str);
        y -= 1;
    }
    // ctx.draw_box(0, 43, 79, 6, white, black);

    // Log
    let log = ecs.fetch::<Gamelog>();
    let mut y = 46;
    for entry in log.entries.iter().rev() {
        if y < 59 {
            ctx.print(2, y, entry);
        }
        y += 1;
    }

    // Tooltips
    draw_tooltips(ecs, ctx);

    // Depth
    let map = ecs.fetch::<Map>();
    let depth = format!("Depth: {depth}", depth = map.depth);
    ctx.print_color(2, PANE_WIDTH + 1, yellow, black, &depth);

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, magenta);
}

/// Draws attribute at fixed x pos and given y pos in light gray
fn draw_attribute(name: &str, attribute: &Attribute, y: i32, ctx: &mut Rltk) {
    let black = RGB::named(rltk::BLACK);
    let attr_gray: RGB = RGB::from_hex("#CCCCCC").expect("Couldn't parse color from hex.");
    let (modifiers, base, bonus) = (attribute.modifiers, attribute.base, attribute.bonus);
    let color = match modifiers.cmp(&0) {
        std::cmp::Ordering::Less => RGB::named(rltk::RED),
        std::cmp::Ordering::Equal => RGB::named(rltk::WHITE),
        std::cmp::Ordering::Greater => RGB::named(rltk::GREEN),
    };

    // Name
    ctx.print_color(50, y, attr_gray, black, name);

    // Total
    ctx.print_color(
        67,
        y,
        color,
        black,
        &format!("{total}", total = base + modifiers),
    );
    // Bonus
    ctx.print_color(73, y, color, black, &format!("{bonus}"));

    // TODO(aalhendi): move glyph to color calc, add ('-')
    if bonus > 0 {
        ctx.set(72, y, color, black, rltk::to_cp437('+'));
    }
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let (min_x, _max_x, min_y, _max_y) = camera::get_screen_bounds(ecs, ctx);
    let white = RGB::named(rltk::WHITE);
    let box_gray: RGB = RGB::from_hex("#999999").expect("Could not parse color from hex");

    let map = ecs.fetch::<Map>();
    let hidden = ecs.read_storage::<Hidden>();
    let attributes = ecs.read_storage::<Attributes>();
    let pools = ecs.read_storage::<Pools>();
    let statuses = ecs.read_storage::<StatusEffect>();
    let durations = ecs.read_storage::<Duration>();
    let names = ecs.read_storage::<Name>();

    let mouse_pos = ctx.mouse_pos();
    let mut mouse_map_pos = mouse_pos;
    // -1 compensate for map being offset from screen
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

    let mouse_idx = map.xy_idx(mouse_map_pos.0, mouse_map_pos.1);
    if !map.visible_tiles[mouse_idx] {
        return;
    }

    // Check if mouse-pos is on an entity, add its TT
    let mut tip_boxes: Vec<Tooltip> = Vec::new();
    spatial::for_each_tile_content(mouse_idx, |entity| {
        if hidden.get(entity).is_some() {
            return;
        }

        let mut tip = Tooltip::new();
        tip.add_line(get_item_display_name(ecs, entity));

        // Comment on attributes
        if let Some(attr) = attributes.get(entity) {
            let mut tip_text = String::new();

            match attr.might.bonus.cmp(&0) {
                std::cmp::Ordering::Less => tip_text += "Weak. ",
                std::cmp::Ordering::Equal => (),
                std::cmp::Ordering::Greater => tip_text += "Strong. ",
            }

            match attr.quickness.bonus.cmp(&0) {
                std::cmp::Ordering::Less => tip_text += "Clumsy. ",
                std::cmp::Ordering::Equal => (),
                std::cmp::Ordering::Greater => tip_text += "Agile. ",
            }

            match attr.fitness.bonus.cmp(&0) {
                std::cmp::Ordering::Less => tip_text += "Unhealthy. ",
                std::cmp::Ordering::Equal => (),
                std::cmp::Ordering::Greater => tip_text += "Healthy. ",
            }

            match attr.intelligence.bonus.cmp(&0) {
                std::cmp::Ordering::Less => tip_text += "Unintelligent. ",
                std::cmp::Ordering::Equal => (),
                std::cmp::Ordering::Greater => tip_text += "Smart. ",
            }

            if tip_text.is_empty() {
                tip_text = "Quite average".to_string();
            }

            tip.add_line(tip_text);
        }

        // Comment on pools
        let stat = pools.get(entity);
        if let Some(stat) = stat {
            tip.add_line(format!("Level: {lvl}", lvl = stat.level));
        }

        // Comment on durations (Status effects)
        for (status, duration, name) in (&statuses, &durations, &names).join() {
            if status.target == entity {
                tip.add_line(format!("{} ({})", name.name, duration.turns));
            }
        }

        tip_boxes.push(tip);
    });

    // No TT on mouse-pos
    if tip_boxes.is_empty() {
        return;
    }

    // Determine if TT renders to right or left of the target
    let arrow_y = mouse_pos.1;
    let (arrow, arrow_x) = if mouse_pos.0 < 40 {
        // Left
        (rltk::to_cp437('→'), mouse_pos.0 - 1)
    } else {
        // Right
        (rltk::to_cp437('←'), mouse_pos.0 + 1)
    };

    ctx.set(arrow_x, arrow_y, white, box_gray, arrow);

    let mut total_height = 0;
    for tt in tip_boxes.iter() {
        total_height += tt.get_height();
    }

    // Shunt all boxes upwards to center the stack
    let mut y = mouse_pos.1 - (total_height as i32 / 2);
    while y + (total_height as i32 / 2) > 50 {
        y -= 1;
    }

    // Draw the boxes
    for tt in tip_boxes.iter() {
        // -2 for border
        let x = if mouse_pos.0 < (PANE_WIDTH - 2) {
            mouse_pos.0 - (1 + tt.get_width() as i32)
        } else {
            mouse_pos.0 + (1 + tt.get_width() as i32)
        };
        tt.render(ctx, x, y);
        y += tt.get_height() as i32;
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
    print_item_menu(ctx, y, 31, count, "Inventory");

    let mut equippable: Vec<Entity> = Vec::new();
    for (j, (entity, _pack)) in (&entities, &backpack)
        .join()
        .filter(|(_entity, item)| item.owned_by(&player_entity))
        .enumerate()
    {
        let label_char = char::from_u32((97 + j) as u32).expect("Invalid char");
        let color = Some(get_item_color(&gs.ecs, entity));
        let name = &get_item_display_name(&gs.ecs, entity);
        print_item_label(ctx, y, label_char, name, color);
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

fn print_item_label(ctx: &mut Rltk, y: i32, label_char: char, name: &String, color: Option<RGB>) {
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

    if let Some(c) = color {
        ctx.print_color(21, y, c, RGB::named(rltk::BLACK), name.to_string());
    } else {
        ctx.print(21, y, name.to_string());
    }
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
        "Rust Roguelike!",
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

fn print_item_menu(ctx: &mut Rltk, y: i32, width: i32, count: usize, label: &str) {
    let yellow = RGB::named(rltk::YELLOW);
    let white = RGB::named(rltk::WHITE);
    let black = RGB::named(rltk::BLACK);
    ctx.draw_box(15, y - 2, width, (count + 3) as i32, white, black);
    ctx.print_color(18, y - 2, yellow, black, label);
    ctx.print_color(18, y + count as i32 + 1, yellow, black, "ESCAPE to cancel");
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

fn draw_hollow_box(
    console: &mut Rltk,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    fg: RGB,
    bg: RGB,
) {
    use rltk::to_cp437;

    console.set(sx, sy, fg, bg, to_cp437('┌'));
    console.set(sx + width, sy, fg, bg, to_cp437('┐'));
    console.set(sx, sy + height, fg, bg, to_cp437('└'));
    console.set(sx + width, sy + height, fg, bg, to_cp437('┘'));
    for x in sx + 1..sx + width {
        console.set(x, sy, fg, bg, to_cp437('─'));
        console.set(x, sy + height, fg, bg, to_cp437('─'));
    }
    for y in sy + 1..sy + height {
        console.set(sx, y, fg, bg, to_cp437('│'));
        console.set(sx + width, y, fg, bg, to_cp437('│'));
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum CheatMenuResult {
    NoResponse,
    Cancel,
    TeleportToExit,
    MagicMapper,
    Heal,
    GodMode,
    GetRich,
}

pub fn show_cheat_mode(_gs: &mut State, ctx: &mut Rltk) -> CheatMenuResult {
    let count = 6;
    let y = 25 - (count / 2);

    print_item_menu(ctx, y, 31, count as usize, "Cheating!");
    print_item_label(ctx, y, 'T', &String::from("Teleport to exit"), None);
    print_item_label(ctx, y + 1, 'M', &String::from("Reveal map"), None);
    print_item_label(ctx, y + 2, 'H', &String::from("Heal all wounds"), None);
    print_item_label(ctx, y + 3, 'G', &String::from("God Mode (No Death)"), None);
    print_item_label(ctx, y + 4, 'L', &String::from("Get Rich (+100g)"), None);

    match ctx.key {
        None => CheatMenuResult::NoResponse,
        Some(key) => match key {
            VirtualKeyCode::T => CheatMenuResult::TeleportToExit,
            VirtualKeyCode::M => CheatMenuResult::MagicMapper,
            VirtualKeyCode::H => CheatMenuResult::Heal,
            VirtualKeyCode::G => CheatMenuResult::GodMode,
            VirtualKeyCode::L => CheatMenuResult::GetRich,
            VirtualKeyCode::Escape => CheatMenuResult::Cancel,
            _ => CheatMenuResult::NoResponse,
        },
    }
}

struct Tooltip {
    lines: Vec<String>,
}

impl Tooltip {
    fn new() -> Tooltip {
        Tooltip { lines: Vec::new() }
    }

    fn add_line<T: Into<String>>(&mut self, line: T) {
        self.lines.push(line.into());
    }

    /// Wrapping not supported, yet
    fn get_width(&self) -> usize {
        self.lines.iter().map(|s| s.len()).max().unwrap_or(0) + 2 // +2 for border
    }

    fn get_height(&self) -> usize {
        self.lines.len() + 2 // +2 for border
    }

    fn render(&self, ctx: &mut Rltk, x: i32, y: i32) {
        let box_gray: RGB = RGB::from_hex("#999999").expect("Oops");
        let light_gray: RGB = RGB::from_hex("#DDDDDD").expect("Oops");
        let white = RGB::named(rltk::WHITE);
        let black = RGB::named(rltk::BLACK);
        ctx.draw_box(
            x,
            y,
            self.get_width() - 1,
            self.get_height() - 1,
            white,
            box_gray,
        );
        for (i, s) in self.lines.iter().enumerate() {
            let col = if i == 0 { white } else { light_gray };
            ctx.print_color(x + 1, y + i as i32 + 1, col, black, s);
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub enum VendorResult {
    NoResponse,
    Cancel,
    Sell,
    BuyMode,
    SellMode,
    Buy,
}
fn vendor_sell_menu(
    gs: &mut State,
    ctx: &mut Rltk,
    _vendor: Entity,
    _mode: VendorMode,
) -> (VendorResult, Option<Entity>, Option<String>, Option<f32>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let items = gs.ecs.read_storage::<Item>();
    let entities = gs.ecs.entities();

    let inventory = (&backpack, &names)
        .join()
        .filter(|item| item.0.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    print_item_menu(
        ctx,
        y,
        51,
        count,
        "Sell Which Item? (space to switch to buy mode)",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    for (j, (entity, _pack, item)) in (&entities, &backpack, &items)
        .join()
        .filter(|item| item.1.owner == *player_entity)
        .enumerate()
    {
        let label_char = char::from_u32((97 + j) as u32).expect("Invalid char");
        let color = Some(get_item_color(&gs.ecs, entity));
        let name = &get_item_display_name(&gs.ecs, entity);
        print_item_label(ctx, y, label_char, name, color);
        ctx.print(50, y, &format!("{val:.1} gp", val = item.base_value * 0.8));
        equippable.push(entity);
        y += 1;
    }

    match ctx.key {
        None => (VendorResult::NoResponse, None, None, None),
        Some(key) => match key {
            VirtualKeyCode::Space => (VendorResult::BuyMode, None, None, None),
            VirtualKeyCode::Escape => (VendorResult::Cancel, None, None, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        VendorResult::Sell,
                        Some(equippable[selection as usize]),
                        None,
                        None,
                    );
                }
                (VendorResult::NoResponse, None, None, None)
            }
        },
    }
}

fn vendor_buy_menu(
    gs: &mut State,
    ctx: &mut Rltk,
    vendor: Entity,
    _mode: VendorMode,
) -> (VendorResult, Option<Entity>, Option<String>, Option<f32>) {
    let vendors = gs.ecs.read_storage::<Vendor>();
    let inventory = get_vendor_items(
        &vendors.get(vendor).unwrap().categories,
        &RAWS.lock().unwrap(),
    );
    let count = inventory.len();

    let mut y = (25 - (count / 2)) as i32;
    print_item_menu(
        ctx,
        y,
        51,
        count,
        "Buy Which Item? (space to switch to sell mode)",
    );

    for (j, sale) in inventory.iter().enumerate() {
        let label_char = char::from_u32((97 + j) as u32).expect("Invalid char");
        print_item_label(ctx, y, label_char, &sale.0, None);
        ctx.print(50, y, &format!("{val:.1} gp", val = sale.1 * 1.2));
        y += 1;
    }

    match ctx.key {
        None => (VendorResult::NoResponse, None, None, None),
        Some(key) => match key {
            VirtualKeyCode::Space => (VendorResult::SellMode, None, None, None),
            VirtualKeyCode::Escape => (VendorResult::Cancel, None, None, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        VendorResult::Buy,
                        None,
                        Some(inventory[selection as usize].0.clone()),
                        Some(inventory[selection as usize].1),
                    );
                }
                (VendorResult::NoResponse, None, None, None)
            }
        },
    }
}

pub fn show_vendor_menu(
    gs: &mut State,
    ctx: &mut Rltk,
    vendor: Entity,
    mode: VendorMode,
) -> (VendorResult, Option<Entity>, Option<String>, Option<f32>) {
    match mode {
        VendorMode::Buy => vendor_buy_menu(gs, ctx, vendor, mode),
        VendorMode::Sell => vendor_sell_menu(gs, ctx, vendor, mode),
    }
}

pub fn get_item_color(ecs: &World, item: Entity) -> RGB {
    let dm = ecs.fetch::<MasterDungeonMap>();
    if let Some(name) = ecs.read_storage::<Name>().get(item) {
        if ecs.read_storage::<CursedItem>().get(item).is_some()
            && dm.identified_items.contains(&name.name)
        {
            return RGB::named(rltk::RED);
        }
    }

    match ecs.read_storage::<MagicItem>().get(item) {
        Some(magic) => match magic.class {
            MagicItemClass::Common => RGB::from_f32(0.5, 1.0, 0.5),
            MagicItemClass::Rare => RGB::from_f32(0.0, 1.0, 1.0),
            MagicItemClass::Legendary => RGB::from_f32(0.71, 0.15, 0.93),
        },
        _ => RGB::from_f32(1.0, 1.0, 1.0),
    }
}

// Outside ECS function
pub fn get_item_display_name(ecs: &World, item: Entity) -> String {
    // Early return for items without a name
    let name_storage = ecs.read_storage::<Name>();
    let name = match name_storage.get(item) {
        Some(name) => name,
        None => return "Nameless item (bug)".to_string(),
    };

    // Non-magic items just return their name
    if ecs.read_storage::<MagicItem>().get(item).is_none() {
        return name.name.clone();
    }

    // For magic items, check if they are identified
    if ecs
        .fetch::<MasterDungeonMap>()
        .identified_items
        .contains(&name.name)
    {
        if let Some(c) = ecs.read_storage::<Consumable>().get(item) {
            if c.max_charges > 1 {
                return format!("{} ({})", name.name.clone(), c.charges).to_string();
            } else {
                return name.name.clone();
            }
        }
    }

    // Return the obfuscated name if available, else a default message
    ecs.read_storage::<ObfuscatedName>()
        .get(item)
        .map(|obfuscated| obfuscated.name.clone())
        .unwrap_or_else(|| "Nameless item (bug)".to_string())
}

pub fn remove_curse_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let items = gs.ecs.read_storage::<Item>();
    let cursed = gs.ecs.read_storage::<CursedItem>();
    let names = gs.ecs.read_storage::<Name>();
    let dm = gs.ecs.fetch::<MasterDungeonMap>();

    let build_cursed_iterator = || {
        (&entities, &items, &cursed)
            .join()
            .filter(|(item_entity, _item, _cursed)| {
                let mut keep = false;
                if let Some(bp) = backpack.get(*item_entity) {
                    if bp.owner == *player_entity {
                        if let Some(name) = names.get(*item_entity) {
                            if dm.identified_items.contains(&name.name) {
                                keep = true;
                            }
                        }
                    }
                }
                // It's equipped, so we know it's cursed
                if let Some(equip) = equipped.get(*item_entity) {
                    if equip.owner == *player_entity {
                        keep = true;
                    }
                }
                keep
            })
    };

    let count = build_cursed_iterator().count();
    let mut y = (25 - (count / 2)) as i32;
    print_item_menu(ctx, y, 31, count, "Remove Curse From Which Item?");

    let mut equippable = Vec::new();
    for (j, (entity, _item, _cursed)) in build_cursed_iterator().enumerate() {
        let label_char = char::from_u32((97 + j) as u32).expect("Invalid char");
        let name = &get_item_display_name(&gs.ecs, entity);
        let color = Some(get_item_color(&gs.ecs, entity));
        print_item_label(ctx, y, label_char, name, color);
        equippable.push(entity);
        y += 1;
    }

    item_menu_input(ctx.key, &equippable, count as i32)
}

pub fn identify_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let items = gs.ecs.read_storage::<Item>();
    let names = gs.ecs.read_storage::<Name>();
    let dm = gs.ecs.fetch::<MasterDungeonMap>();
    let obfuscated = gs.ecs.read_storage::<ObfuscatedName>();

    let build_cursed_iterator = || {
        (&entities, &items).join().filter(|(item_entity, _item)| {
            let mut keep = false;
            if let Some(bp) = backpack.get(*item_entity) {
                if bp.owner == *player_entity {
                    if let Some(name) = names.get(*item_entity) {
                        if obfuscated.get(*item_entity).is_some()
                            && !dm.identified_items.contains(&name.name)
                        {
                            keep = true;
                        }
                    }
                }
            }
            // It's equipped, so we know it's cursed
            if let Some(equip) = equipped.get(*item_entity) {
                if equip.owner == *player_entity {
                    if let Some(name) = names.get(*item_entity) {
                        if obfuscated.get(*item_entity).is_some()
                            && !dm.identified_items.contains(&name.name)
                        {
                            keep = true;
                        }
                    }
                }
            }
            keep
        })
    };

    let count = build_cursed_iterator().count();

    let mut y = (25 - (count / 2)) as i32;
    print_item_menu(ctx, y, 31, count, "Identify Which Item?");

    let mut equippable = Vec::new();
    for (j, (entity, _item)) in build_cursed_iterator().enumerate() {
        let label_char = char::from_u32((97 + j) as u32).expect("Invalid char");
        let name = &get_item_display_name(&gs.ecs, entity);
        let color = Some(get_item_color(&gs.ecs, entity));
        print_item_label(ctx, y, label_char, name, color);
        equippable.push(entity);
        y += 1;
    }

    item_menu_input(ctx.key, &equippable, count as i32)
}
