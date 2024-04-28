use rltk::{Rltk, VirtualKeyCode};
use specs::{Entity, WorldExt, Join};

use crate::{gui::item_render::{get_item_color, get_item_display_name}, raws::{rawsmaster::get_vendor_items, RAWS}, InBackpack, Item, Name, State, Vendor, VendorMode};

use super::{print_item_label, print_item_menu};

#[derive(PartialEq, Copy, Clone)]
pub enum VendorResult {
    NoResponse,
    Cancel,
    Sell,
    BuyMode,
    SellMode,
    Buy,
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

