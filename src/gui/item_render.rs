use rltk::RGB;
use specs::{Entity, World, WorldExt};

use crate::{
    dungeon::MasterDungeonMap, Consumable, CursedItem, MagicItem, MagicItemClass, Name,
    ObfuscatedName,
};

pub fn get_item_color(ecs: &World, item: Entity) -> RGB {
    let dm = ecs.fetch::<MasterDungeonMap>();
    if let Some(name) = ecs.read_storage::<Name>().get(item) {
        if ecs.read_storage::<CursedItem>().get(item).is_some()
            && dm.identified_items.contains(&name.name)
        {
            return RGB::named(rltk::RED);
        }
    }

    match ecs.read_storage::<MagicItem>().get(item) {
        Some(magic) => match magic.class {
            MagicItemClass::Common => RGB::from_f32(0.5, 1.0, 0.5),
            MagicItemClass::Rare => RGB::from_f32(0.0, 1.0, 1.0),
            MagicItemClass::Legendary => RGB::from_f32(0.71, 0.15, 0.93),
        },
        _ => RGB::from_f32(1.0, 1.0, 1.0),
    }
}

// Outside ECS function
pub fn get_item_display_name(ecs: &World, item: Entity) -> String {
    // Early return for items without a name
    let name = if let Some(name) = ecs.read_storage::<Name>().get(item) {
        name.name.clone()
    } else {
        return "Nameless item (bug)".to_string();
    };

    // Non-magic items just return their name
    if ecs.read_storage::<MagicItem>().get(item).is_none() {
        return name;
    }

    // For magic items, check if they are identified
    let dm = ecs.fetch::<MasterDungeonMap>();
    if dm.identified_items.contains(&name) {
        if let Some(c) = ecs.read_storage::<Consumable>().get(item) {
            if c.max_charges > 1 {
                return format!("{} ({})", name, c.charges);
            }
        }
        return name;
    }

    // Return the obfuscated name if available, else a default message
    if let Some(obfuscated) = ecs.read_storage::<ObfuscatedName>().get(item) {
        obfuscated.name.clone()
    } else {
        "Unidentified magic item".to_string()
    }
}
