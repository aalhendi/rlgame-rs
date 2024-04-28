use rltk::Rltk;
use specs::{Entity, WorldExt, Join};

use crate::{dungeon::MasterDungeonMap, gui::item_render::{get_item_color, get_item_display_name}, Equipped, InBackpack, Item, Name, ObfuscatedName, State};

use super::{item_menu_input, print_item_label, print_item_menu, ItemMenuResult};

pub fn identify_menu(gs: &mut State, ctx: &mut Rltk) -> (ItemMenuResult, Option<Entity>) {
    let player_entity = gs.ecs.fetch::<Entity>();
    let equipped = gs.ecs.read_storage::<Equipped>();
    let backpack = gs.ecs.read_storage::<InBackpack>();
    let entities = gs.ecs.entities();
    let items = gs.ecs.read_storage::<Item>();
    let names = gs.ecs.read_storage::<Name>();
    let dm = gs.ecs.fetch::<MasterDungeonMap>();
    let obfuscated = gs.ecs.read_storage::<ObfuscatedName>();

    let build_cursed_iterator = || {
        (&entities, &items).join().filter(|(item_entity, _item)| {
            let mut keep = false;
            if let Some(bp) = backpack.get(*item_entity) {
                if bp.owner == *player_entity {
                    if let Some(name) = names.get(*item_entity) {
                        if obfuscated.get(*item_entity).is_some()
                            && !dm.identified_items.contains(&name.name)
                        {
                            keep = true;
                        }
                    }
                }
            }
            // It's equipped, so we know it's cursed
            if let Some(equip) = equipped.get(*item_entity) {
                if equip.owner == *player_entity {
                    if let Some(name) = names.get(*item_entity) {
                        if obfuscated.get(*item_entity).is_some()
                            && !dm.identified_items.contains(&name.name)
                        {
                            keep = true;
                        }
                    }
                }
            }
            keep
        })
    };

    let count = build_cursed_iterator().count();

    let mut y = (25 - (count / 2)) as i32;
    print_item_menu(ctx, y, 31, count, "Identify Which Item?");

    let mut equippable = Vec::new();
    for (j, (entity, _item)) in build_cursed_iterator().enumerate() {
        let label_char = char::from_u32((97 + j) as u32).expect("Invalid char");
        let name = &get_item_display_name(&gs.ecs, entity);
        let color = Some(get_item_color(&gs.ecs, entity));
        print_item_label(ctx, y, label_char, name, color);
        equippable.push(entity);
        y += 1;
    }

    item_menu_input(ctx.key, &equippable, count as i32)
}
