use rltk::{ColorPair, DrawBatch, Rltk, VirtualKeyCode, RGB};

use crate::{saveload_system, RunState, State};

use super::print_menu_item;

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
    let mut draw_batch = DrawBatch::new();
    let save_exists = saveload_system::save_exists();
    let runstate = gs.ecs.fetch::<RunState>();

    draw_batch.print_color_centered(
        15,
        "Rust Roguelike!",
        ColorPair::new(RGB::named(rltk::YELLOW), RGB::named(rltk::BLACK)),
    );

    if let RunState::MainMenu {
        menu_selection: cur_hovering,
    } = *runstate
    {
        print_menu_item(
            &mut draw_batch,
            "Begin New Game",
            24,
            cur_hovering == MainMenuSelection::NewGame,
        );

        if save_exists {
            print_menu_item(
                &mut draw_batch,
                "Load Game",
                25,
                cur_hovering == MainMenuSelection::LoadGame,
            );
        }
        print_menu_item(
            &mut draw_batch,
            "Quit",
            26,
            cur_hovering == MainMenuSelection::Quit,
        );

        let _ = draw_batch.submit(6000);

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
