use rltk::{Rltk, RGB};

use crate::gamelog::events::get_event_count;

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    let yellow = RGB::named(rltk::YELLOW);
    let black = RGB::named(rltk::BLACK);
    let white = RGB::named(rltk::WHITE);
    let magenta = RGB::named(rltk::MAGENTA);

    ctx.print_color_centered(15, yellow, black, "You Died!");
    ctx.print_color_centered(18, white, black, "Some day there might be stats here...");

    let turns_txt = &format!("You lived for {} turns.", get_event_count("Turn"));
    ctx.print_color_centered(19, white, black, turns_txt);

    let dmg_out_txt = &format!(
        "You inflicted {} points of damage.",
        get_event_count("Damage Inflicted")
    );
    ctx.print_color_centered(20, white, black, dmg_out_txt);

    let dmg_in_txt = &format!(
        "You suffered {} points of damage.",
        get_event_count("Damage Taken")
    );
    ctx.print_color_centered(21, white, black, dmg_in_txt);

    ctx.print_color_centered(23, magenta, black, "Press any key to return to the menu.");

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu,
    }
}