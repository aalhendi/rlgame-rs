use rltk::{DrawBatch, Rltk, VirtualKeyCode};

use crate::State;

use super::{print_item_label, print_item_menu};

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
    let mut draw_batch = DrawBatch::new();
    let count = 6;
    let y = 25 - (count / 2);

    print_item_menu(&mut draw_batch, y, 31, count as usize, "Cheating!");
    print_item_label(
        &mut draw_batch,
        y,
        'T',
        &String::from("Teleport to exit"),
        None,
    );
    print_item_label(
        &mut draw_batch,
        y + 1,
        'M',
        &String::from("Reveal map"),
        None,
    );
    print_item_label(
        &mut draw_batch,
        y + 2,
        'H',
        &String::from("Heal all wounds"),
        None,
    );
    print_item_label(
        &mut draw_batch,
        y + 3,
        'G',
        &String::from("God Mode (No Death)"),
        None,
    );
    print_item_label(
        &mut draw_batch,
        y + 4,
        'L',
        &String::from("Get Rich (+100g)"),
        None,
    );

    let _ = draw_batch.submit(6000);

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
