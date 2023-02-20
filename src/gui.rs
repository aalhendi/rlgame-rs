use super::{CombatStats, Player, MAPHEIGHT, MAPWIDTH};
use rltk::{Rltk, RGB};
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

    let yellow = RGB::named(rltk::YELLOW);
    let black = RGB::named(rltk::BLACK);
    let red = RGB::named(rltk::RED);

    for (_player, stats) in (&players, &combat_stats).join() {
        let health = format!(
            " HP: {hp} / {max_hp} ",
            hp = stats.hp,
            max_hp = stats.max_hp
        );
        ctx.print_color(
            12,
            43,
            yellow,
            black,
            &health,
        );

        ctx.draw_bar_horizontal(
            28,
            43,
            51,
            stats.hp,
            stats.max_hp,
            red,
            black,
        );
    }
}
