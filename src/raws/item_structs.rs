use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Item {
    pub name: String,
    pub renderable: Option<Renderable>,
    pub consumable: Option<Consumable>,
    pub weapon: Option<Weapon>,
    pub shield: Option<Shield>,
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
    pub power_bonus: i32,
}

#[derive(Deserialize, Debug)]
pub struct Shield {
    pub defense_bonus: i32,
}
