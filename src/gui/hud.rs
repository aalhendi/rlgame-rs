use std::cmp::Ordering;

use rltk::{Point, Rltk, TextBlock, RGB};
use specs::{Entity, Join, World, WorldExt};

use crate::{map::camera::PANE_WIDTH, gamelog, gui::{item_render::{get_item_color, get_item_display_name}, tooltips}, Attribute, Attributes, Consumable, Duration, Equipped, HungerClock, HungerState, InBackpack, KnownSpells, Map, Name, Pools, StatusEffect, Weapon};

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
    let weapons = ecs.read_storage::<Weapon>();
    for (entity, equipped_by) in (&entities, &equipped).join() {
        if equipped_by.owner == *player_entity {
            let name = &get_item_display_name(ecs, entity);
            ctx.print_color(50, y, get_item_color(ecs, entity), black, name);
            y += 1;

            if let Some(weapon) = weapons.get(entity) {
                let mut weapon_info = match weapon.damage_bonus.cmp(&0) {
                    Ordering::Less => format!(
                        "┤ {} ({}d{}{})",
                        &name, weapon.damage_n_dice, weapon.damage_die_type, weapon.damage_bonus
                    ),
                    Ordering::Equal => format!(
                        "┤ {} ({}d{})",
                        &name, weapon.damage_n_dice, weapon.damage_die_type
                    ),
                    Ordering::Greater => format!(
                        "┤ {} ({}d{}+{})",
                        &name, weapon.damage_n_dice, weapon.damage_die_type, weapon.damage_bonus
                    ),
                };

                if let Some(range) = weapon.range {
                    weapon_info += &format!(" (range: {range}, F to fire, V cycle targets)");
                }
                weapon_info += " ├";
                ctx.print_color(3, 45, yellow, black, &weapon_info);
            }
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
            ctx.print_color(50, y, yellow, black, format!("↑{index}"));
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
        ctx.print_color(50, y, cyan, black, format!("^{idx}", idx = idx + 1));
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

    // Draw the log
    let block = TextBlock::new(1, 46, 79, 58);
    gamelog::print_log(
        &mut rltk::BACKEND_INTERNAL.lock().consoles[1].console,
        Point::new(1, 23),
    );
    block.render(&mut rltk::BACKEND_INTERNAL.lock().consoles[0].console);

    // Tooltips
    tooltips::draw_tooltips(ecs, ctx);

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
        Ordering::Less => RGB::named(rltk::RED),
        Ordering::Equal => RGB::named(rltk::WHITE),
        Ordering::Greater => RGB::named(rltk::GREEN),
    };

    // Name
    ctx.print_color(50, y, attr_gray, black, name);

    // Total
    ctx.print_color(
        67,
        y,
        color,
        black,
        format!("{total}", total = base + modifiers),
    );
    // Bonus
    let var_name = format!("{bonus}");
    ctx.print_color(73, y, color, black, &var_name);

    // TODO(aalhendi): move glyph to color calc, add ('-')
    if bonus > 0 {
        ctx.set(72, y, color, black, rltk::to_cp437('+'));
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