use super::{
    gamelog::Gamelog, CombatStats, InBackpack, Map, Name, Player, Position, State, MAPHEIGHT,
    MAPWIDTH,
};
use rltk::{Point, Rltk, VirtualKeyCode, RGB};
use specs::prelude::*;

pub fn draw_ui(ecs: &World, ctx: &mut Rltk) {
    ctx.draw_box(
        0,
        MAPHEIGHT,
        MAPWIDTH - 1,
        6,
        RGB::named(rltk::WHITE),
        RGB::named(rltk::BLACK),
    );

    // TODO: If player is a resource in the ECS, can't we just fetch it insead of
    // player entity and combat_stats component read calls?
    let combat_stats = ecs.read_storage::<CombatStats>();
    let players = ecs.read_storage::<Player>();
    let log = ecs.fetch::<Gamelog>();

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

    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(
            " HP: {hp} / {max_hp} ",
            hp = stats.hp,
            max_hp = stats.max_hp
        );
        ctx.print_color(12, 43, yellow, black, &health);

        ctx.draw_bar_horizontal(28, 43, 51, stats.hp, stats.max_hp, red, black);
    }

    // Draw mouse cursor
    let mouse_pos = ctx.mouse_pos();
    ctx.set_bg(mouse_pos.0, mouse_pos.1, magenta);
    draw_tooltips(ecs, ctx);
}

fn draw_tooltips(ecs: &World, ctx: &mut Rltk) {
    let white = RGB::named(rltk::WHITE);
    let grey = RGB::named(rltk::GREY);

    let map = ecs.fetch::<Map>();
    let names = ecs.read_storage::<Name>();
    let positions = ecs.read_storage::<Position>();

    let mouse_pos = ctx.mouse_pos();
    // Check if mouse is on map
    if mouse_pos.0 >= map.width || mouse_pos.1 >= map.height {
        return;
    }
    let mut tooltip: Vec<String> = Vec::new();

    for (name, pos) in (&names, &positions).join() {
        let idx = map.xy_idx(pos.x, pos.y);
        if pos.x == mouse_pos.0 && pos.y == mouse_pos.1 && map.visible_tiles[idx] {
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

pub fn show_inventory(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();

    let white = RGB::named(rltk::WHITE);

    let inventory = (&backpack, &names)
        .join()
        .filter(|(item, _name)| item.owner == *player_entity);
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    ctx.draw_box(
        15,
        y - 2,
        31,
        (count + 3) as i32,
        white,
        RGB::named(rltk::BLACK),
    );
    ctx.print_color(
        18,
        y - 2,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "Inventory",
    );
    ctx.print_color(
        18,
        y + count as i32 + 1,
        RGB::named(rltk::YELLOW),
        RGB::named(rltk::BLACK),
        "ESCAPE to cancel",
    );

    let mut equippable: Vec<Entity> = Vec::new();
    for (j, (entity, _pack, item_name)) in (&entities, &backpack, &names)
        .join()
        .filter(|(_entity, item, _name)| item.owner == *player_entity)
        .enumerate()
    {
let label_char = char::from_u32((97 + j) as u32).expect("Invalid char");
        print_item_label(ctx, y, label_char, item_name);
        equippable.push(entity);
        y += 1;
    }

    // TODO: Replace with if-let
    match ctx.key {
        None => (ItemMenuResult::NoResponse, None),
        Some(key) => match key {
            VirtualKeyCode::Escape => (ItemMenuResult::Cancel, None),
            _ => {
                let selection = rltk::letter_to_option(key);
                if selection > -1 && selection < count as i32 {
                    return (
                        ItemMenuResult::Selected,
                        Some(equippable[selection as usize]),
                    );
                }
                (ItemMenuResult::NoResponse, None)
            }
        },
    }
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
