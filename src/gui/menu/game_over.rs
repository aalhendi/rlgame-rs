use rltk::{ColorPair, DrawBatch, Rltk, RGB};

use crate::gamelog::events::get_event_count;

#[derive(PartialEq, Copy, Clone)]
pub enum GameOverResult {
    NoSelection,
    QuitToMenu,
}

pub fn game_over(ctx: &mut Rltk) -> GameOverResult {
    let mut draw_batch = DrawBatch::new();

    let yellow = RGB::named(rltk::YELLOW);
    let black = RGB::named(rltk::BLACK);
    let white = RGB::named(rltk::WHITE);
    let magenta = RGB::named(rltk::MAGENTA);

    draw_batch.print_color_centered(15, "You Died!", ColorPair::new(yellow, black));
    draw_batch.print_color_centered(
        18,
        "Some day there might be stats here...",
        ColorPair::new(white, black),
    );

    let turns_txt = &format!("You lived for {} turns.", get_event_count("Turn"));
    draw_batch.print_color_centered(19, turns_txt, ColorPair::new(white, black));

    let dmg_out_txt = &format!(
        "You inflicted {} points of damage.",
        get_event_count("Damage Inflicted")
    );
    draw_batch.print_color_centered(20, dmg_out_txt, ColorPair::new(white, black));

    let dmg_in_txt = &format!(
        "You suffered {} points of damage.",
        get_event_count("Damage Taken")
    );
    draw_batch.print_color_centered(21, dmg_in_txt, ColorPair::new(white, black));

    draw_batch.print_color_centered(
        23,
        "Press any key to return to the menu.",
        ColorPair::new(magenta, black),
    );

    let _ = draw_batch.submit(6000);

    match ctx.key {
        None => GameOverResult::NoSelection,
        Some(_) => GameOverResult::QuitToMenu,
    }
}
