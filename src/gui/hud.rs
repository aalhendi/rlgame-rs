use std::cmp::Ordering;

use rltk::{ColorPair, DrawBatch, Point, Rltk, TextBlock, RGB};
use specs::{Entity, Join, World, WorldExt};

use crate::{
    gamelog,
    gui::{
        item_render::{get_item_color, get_item_display_name},
        tooltips,
    },
    map::camera::PANE_WIDTH,
    Attribute, Attributes, Consumable, Duration, Equipped, HungerClock, HungerState, InBackpack,
    KnownSpells, Map, Name, Pools, StatusEffect, Weapon,
};

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    use rltk::to_cp437;
    let mut draw_batch = DrawBatch::new();

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
    let box_color_pair = ColorPair::new(box_gray, black);
    let white_color_pair = ColorPair::new(white, black);

    // Overall box
    draw_hollow_box(&mut draw_batch, 0, 0, 79, 59, box_color_pair);
    // Map box
    draw_hollow_box(&mut draw_batch, 0, 0, 49, 45, box_color_pair);
    // Log box
    draw_hollow_box(&mut draw_batch, 0, 45, 79, 14, box_color_pair);
    // Top-right (Attributes) panel
    draw_hollow_box(&mut draw_batch, 49, 0, 30, 8, box_color_pair);

    // Box connectors for style
    draw_batch.set(Point::new(0, 45), box_color_pair, to_cp437('├'));
    draw_batch.set(Point::new(49, 8), box_color_pair, to_cp437('├'));
    draw_batch.set(Point::new(49, 0), box_color_pair, to_cp437('┬'));
    draw_batch.set(Point::new(49, 45), box_color_pair, to_cp437('┴'));
    draw_batch.set(Point::new(79, 8), box_color_pair, to_cp437('┤'));
    draw_batch.set(Point::new(79, 45), box_color_pair, to_cp437('┤'));

    // Town Name
    let map = ecs.fetch::<Map>();
    let name_length = (map.name.len() + 2) as i32; // +2 for wrapping char
    let x_pos = PANE_WIDTH - (name_length / 2);
    // Endcap / wrapping chars
    draw_batch.set(Point::new(x_pos, 0), box_color_pair, to_cp437('┤'));
    draw_batch.set(
        Point::new(x_pos + name_length - 1, 0),
        box_color_pair,
        to_cp437('├'),
    );
    draw_batch.print_color(Point::new(x_pos + 1, 0), &map.name, white_color_pair);
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

    draw_batch.print_color(Point::new(50, 1), health, white_color_pair);
    draw_batch.print_color(Point::new(50, 2), mana, white_color_pair);
    draw_batch.print_color(Point::new(50, 3), level, white_color_pair);
    draw_batch.bar_horizontal(
        Point::new(64, 1),
        14,
        health_curr,
        health_max,
        ColorPair::new(red, black),
    );
    draw_batch.bar_horizontal(
        Point::new(64, 2),
        14,
        mana_curr,
        mana_max,
        ColorPair::new(blue, black),
    );
    draw_batch.bar_horizontal(
        Point::new(64, 3),
        14,
        xp_level_curr,
        1000,
        ColorPair::new(gold, black),
    );

    // Attributes
    let attributes = ecs.read_storage::<Attributes>();
    let p_attr = attributes.get(*player_entity).unwrap();
    draw_attribute("Might:", &p_attr.might, 4, &mut draw_batch);
    draw_attribute("Quickness:", &p_attr.quickness, 5, &mut draw_batch);
    draw_attribute("Fitness:", &p_attr.fitness, 6, &mut draw_batch);
    draw_attribute("Intelligence:", &p_attr.intelligence, 7, &mut draw_batch);

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
    draw_batch.print_color(Point::new(50, 9), weight_str, white_color_pair);
    draw_batch.print_color(Point::new(50, 10), initiative_str, white_color_pair);

    // Gold
    let gold_str = &format!("Gold: {amt:.1}", amt = player_pools.gold);
    draw_batch.print_color(Point::new(50, 11), gold_str, ColorPair::new(gold, black));

    // Wearables / Equipped
    let mut y = 13; // Starting pt
    let equipped = ecs.read_storage::<Equipped>();
    let weapons = ecs.read_storage::<Weapon>();
    for (entity, equipped_by) in (&entities, &equipped).join() {
        if equipped_by.owner == *player_entity {
            let name = &get_item_display_name(ecs, entity);
            draw_batch.print_color(
                Point::new(50, y),
                name,
                ColorPair::new(get_item_color(ecs, entity), black),
            );
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
                draw_batch.print_color(
                    Point::new(3, 45),
                    &weapon_info,
                    ColorPair::new(yellow, black),
                );
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
            draw_batch.print_color(
                Point::new(50, y),
                format!("↑{index}"),
                ColorPair::new(yellow, black),
            );
            draw_batch.print_color(
                Point::new(53, y),
                name,
                ColorPair::new(get_item_color(ecs, entity), black),
            );
            y += 1;
            index += 1;
        }
    }

    // Spells
    y += 1;
    let known_spells_storage = ecs.read_storage::<KnownSpells>();
    let known_spells = &known_spells_storage.get(*player_entity).unwrap().spells;
    for (idx, spell) in known_spells.iter().enumerate() {
        draw_batch.print_color(
            Point::new(50, y),
            format!("^{idx}", idx = idx + 1),
            ColorPair::new(cyan, black),
        );
        let spell_str = &format!("{} ({})", spell.display_name, spell.mana_cost);
        draw_batch.print_color(Point::new(53, y), spell_str, ColorPair::new(cyan, black));
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
        HungerState::WellFed => {
            draw_batch.print_color(Point::new(50, y), "Well Fed", ColorPair::new(green, black));
        }
        HungerState::Normal => {
            draw_batch.print_color(Point::new(50, y), "Normal", ColorPair::new(white, black));
        }
        HungerState::Hungry => {
            draw_batch.print_color(Point::new(50, y), "Hungry", ColorPair::new(orange, black));
        }
        HungerState::Starving => {
            draw_batch.print_color(Point::new(50, y), "Starving", ColorPair::new(red, black));
        }
    }
    if !matches!(hc.state, HungerState::Normal) {
        y -= 1;
    }

    for (status, duration, name) in (&statuses, &durations, &names).join() {
        if status.target != *player_entity {
            continue;
        }

        let duration_str = &format!("{} ({})", name.name, duration.turns);
        draw_batch.print_color(Point::new(50, y), duration_str, ColorPair::new(red, black));
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
    draw_batch.print_color(
        Point::new(2, PANE_WIDTH + 1),
        &depth,
        ColorPair::new(yellow, black),
    );

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    let mouse_pos = Point::new(mouse_pos.0, mouse_pos.1);
    draw_batch.set_bg(mouse_pos, magenta);

    let _ = draw_batch.submit(5000);
}

/// Draws attribute at fixed x pos and given y pos in light gray
fn draw_attribute(name: &str, attribute: &Attribute, y: i32, draw_batch: &mut DrawBatch) {
    let black = RGB::named(rltk::BLACK);
    let attr_gray: RGB = RGB::from_hex("#CCCCCC").expect("Couldn't parse color from hex.");
    let (modifiers, base, bonus) = (attribute.modifiers, attribute.base, attribute.bonus);
    let color = match modifiers.cmp(&0) {
        Ordering::Less => RGB::named(rltk::RED),
        Ordering::Equal => RGB::named(rltk::WHITE),
        Ordering::Greater => RGB::named(rltk::GREEN),
    };

    // Name
    draw_batch.print_color(Point::new(50, y), name, ColorPair::new(attr_gray, black));

    // Total
    draw_batch.print_color(
        Point::new(67, y),
        format!("{total}", total = base + modifiers),
        ColorPair::new(color, black),
    );
    // Bonus
    let var_name = format!("{bonus}");
    draw_batch.print_color(Point::new(73, y), &var_name, ColorPair::new(color, black));

    // TODO(aalhendi): move glyph to color calc, add ('-')
    if bonus > 0 {
        draw_batch.set(
            Point::new(72, y),
            ColorPair::new(color, black),
            rltk::to_cp437('+'),
        );
    }
}

fn draw_hollow_box(
    draw_batch: &mut DrawBatch,
    sx: i32,
    sy: i32,
    width: i32,
    height: i32,
    color_pair: ColorPair,
) {
    use rltk::to_cp437;

    draw_batch.set(Point::new(sx, sy), color_pair, to_cp437('┌'));
    draw_batch.set(Point::new(sx + width, sy), color_pair, to_cp437('┐'));
    draw_batch.set(Point::new(sx, sy + height), color_pair, to_cp437('└'));
    draw_batch.set(
        Point::new(sx + width, sy + height),
        color_pair,
        to_cp437('┘'),
    );
    for x in sx + 1..sx + width {
        draw_batch.set(Point::new(x, sy), color_pair, to_cp437('─'));
        draw_batch.set(Point::new(x, sy + height), color_pair, to_cp437('─'));
    }
    for y in sy + 1..sy + height {
        draw_batch.set(Point::new(sx, y), color_pair, to_cp437('│'));
        draw_batch.set(Point::new(sx + width, y), color_pair, to_cp437('│'));
    }
}
