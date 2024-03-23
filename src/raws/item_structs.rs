use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Item {
    pub name: String,
    pub renderable: Option<Renderable>,
    pub consumable: Option<Consumable>,
    pub weapon: Option<Weapon>,
    pub wearable: Option<Wearable>,
    pub initiative_penalty: Option<f32>,
    pub weight_lbs: Option<f32>,
    pub base_value: Option<f32>,
    pub vendor_category: Option<String>,
    pub magic: Option<MagicItem>,
}

#[derive(Deserialize, Debug)]
pub struct MagicItem {
    pub class: String,
    pub naming: String,
    pub cursed: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct Renderable {
    pub glyph: String,
    pub fg: String,
    pub bg: String,
    pub order: i32,
}

#[derive(Deserialize, Debug)]
pub struct Consumable {
    pub effects: HashMap<String, String>, // effect_name, effect_value
}

// TODO: Use an equipment_slot field in spawns.json and have an enum for equippables under Item struct. Makes it easier to expand equip slots to amulets, rings, etc.
#[derive(Deserialize, Debug)]
pub struct Weapon {
    pub range: String,
    pub attribute: String,
    pub base_damage: String,
    pub hit_bonus: i32,
}

#[derive(Deserialize, Debug)]
pub struct Wearable {
    pub armor_class: f32,
    pub slot: String,
}
