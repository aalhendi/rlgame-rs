use rltk::{DrawBatch, Rltk};
use specs::{Entity, Join, WorldExt};

use crate::{
    dungeon::MasterDungeonMap,
    gui::item_render::{get_item_color, get_item_display_name},
    CursedItem, Equipped, InBackpack, Item, Name, State,
};

use super::{item_menu_input, print_item_label, print_item_menu, ItemMenuResult};

pub fn remove_curse_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let mut draw_batch = DrawBatch::new();

    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let items = gs.ecs.read_storage::<Item>();
    let cursed = gs.ecs.read_storage::<CursedItem>();
    let names = gs.ecs.read_storage::<Name>();
    let dm = gs.ecs.fetch::<MasterDungeonMap>();

    let build_cursed_iterator = || {
        (&entities, &items, &cursed)
            .join()
            .filter(|(item_entity, _item, _cursed)| {
                let mut keep = false;
                if let Some(bp) = backpack.get(*item_entity) {
                    if bp.owner == *player_entity {
                        if let Some(name) = names.get(*item_entity) {
                            if dm.identified_items.contains(&name.name) {
                                keep = true;
                            }
                        }
                    }
                }
                // It's equipped, so we know it's cursed
                if let Some(equip) = equipped.get(*item_entity) {
                    if equip.owner == *player_entity {
                        keep = true;
                    }
                }
                keep
            })
    };

    let count = build_cursed_iterator().count();
    let mut y = (25 - (count / 2)) as i32;
    print_item_menu(
        &mut draw_batch,
        y,
        31,
        count,
        "Remove Curse From Which Item?",
    );

    let mut equippable = Vec::new();
    for (j, (entity, _item, _cursed)) in build_cursed_iterator().enumerate() {
        let label_char = char::from_u32((97 + j) as u32).expect("Invalid char");
        let name = &get_item_display_name(&gs.ecs, entity);
        let color = Some(get_item_color(&gs.ecs, entity));
        print_item_label(&mut draw_batch, y, label_char, name, color);
        equippable.push(entity);
        y += 1;
    }

    let _ = draw_batch.submit(6000);
    item_menu_input(ctx.key, &equippable, count as i32)
}
