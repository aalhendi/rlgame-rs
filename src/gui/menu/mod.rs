pub mod cheat;
pub mod game_over;
pub mod identify;
pub mod main_menu;
pub mod ranged_target;
pub mod remove_curse;
pub mod vendor;

use rltk::{ColorPair, DrawBatch, Point, Rect, Rltk, VirtualKeyCode, RGB};
use specs::{Component, Entity, Join, WorldExt};

use crate::{Equipped, InBackpack, Name, Owned, State};

use super::item_render::{get_item_color, get_item_display_name};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ItemMenuResult {
    Cancel,
    NoResponse,
    Selected,
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

pub fn show_menu<T: Owned + Component>(
    gs: &mut State,
    ctx: &mut Rltk,
) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let names = gs.ecs.read_storage::<Name>();
    let backpack = gs.ecs.read_storage::<T>();
    let entities = gs.ecs.entities();

    let mut draw_batch = DrawBatch::new();

    let inventory = (&backpack, &names)
        .join()
        .filter(|(item, _name)| item.owned_by(&player_entity));
    let count = inventory.count();

    let mut y = (25 - (count / 2)) as i32;
    print_item_menu(&mut draw_batch, y, 31, count, "Inventory");

    let mut equippable: Vec<Entity> = Vec::new();
    for (j, (entity, _pack)) in (&entities, &backpack)
        .join()
        .filter(|(_entity, item)| item.owned_by(&player_entity))
        .enumerate()
    {
        let label_char = char::from_u32((97 + j) as u32).expect("Invalid char");
        let color = Some(get_item_color(&gs.ecs, entity));
        let name = &get_item_display_name(&gs.ecs, entity);
        print_item_label(&mut draw_batch, y, label_char, name, color);
        equippable.push(entity);
        y += 1;
    }

    let _ = draw_batch.submit(6000);
    item_menu_input(ctx.key, &equippable, count as i32)
}

fn print_menu_item(draw_batch: &mut DrawBatch, text: &str, y: i32, is_highlighted: bool) {
    let fg = {
        if is_highlighted {
            RGB::named(rltk::MAGENTA)
        } else {
            RGB::named(rltk::WHITE)
        }
    };
    draw_batch.print_color_centered(y, text, ColorPair::new(fg, RGB::named(rltk::BLACK)));
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

fn print_item_label(
    draw_batch: &mut DrawBatch,
    y: i32,
    label_char: char,
    name: &String,
    color: Option<RGB>,
) {
    draw_batch.set(
        Point::new(17, y),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        rltk::to_cp437('('),
    );
    draw_batch.set(
        Point::new(18, y),
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
        rltk::to_cp437(label_char),
    );
    draw_batch.set(
        Point::new(19, y),
        ColorPair::new(RGB::named(rltk::WHITE), RGB::named(rltk::BLACK)),
        rltk::to_cp437(')'),
    );

    if let Some(c) = color {
        draw_batch.print_color(
            Point::new(21, y),
            name.to_string(),
            ColorPair::new(c, RGB::named(rltk::BLACK)),
        );
    } else {
        draw_batch.print(Point::new(21, y), name.to_string());
    }
}

fn print_item_menu(draw_batch: &mut DrawBatch, y: i32, width: i32, count: usize, label: &str) {
    let yellow = RGB::named(rltk::YELLOW);
    let white = RGB::named(rltk::WHITE);
    let black = RGB::named(rltk::BLACK);
    let rect = Rect::with_size(15, y - 2, width, (count + 3) as i32);
    draw_batch.draw_box(rect, ColorPair::new(white, black));
    draw_batch.print_color(Point::new(18, y - 2), label, ColorPair::new(yellow, black));
    draw_batch.print_color(
        Point::new(18, y + count as i32 + 1),
        "ESCAPE to cancel",
        ColorPair::new(yellow, black),
    );
}
