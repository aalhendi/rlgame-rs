use crate::{dungeon::MasterDungeonMap, MagicItem, ObfuscatedName};

use super::Name;

use specs::prelude::*;
pub mod collection_system;
pub mod drop_system;
pub mod identification_system;
pub mod remove_system;
pub mod use_equip;
pub mod use_system;

// Inside ECS function
pub fn obfuscate_name(
    item: Entity,
    names: &ReadStorage<Name>,
    magic_items: &ReadStorage<MagicItem>,
    obfuscated_names: &ReadStorage<ObfuscatedName>,
    dm: &MasterDungeonMap,
) -> String {
    // Early return for items without a name
    let name = match names.get(item) {
        Some(name) => name,
        None => return "Nameless item (bug)".to_string(),
    };

    // Non-magic items just return their name
    if magic_items.get(item).is_none() {
        return name.name.clone();
    }

    // For magic items, check if they are identified
    if dm.identified_items.contains(&name.name) {
        return name.name.clone();
    }

    // Return the obfuscated name if available, else a default message
    obfuscated_names
        .get(item)
        .map(|obfuscated| obfuscated.name.clone())
        .unwrap_or_else(|| "Nameless item (bug)".to_string())
}
