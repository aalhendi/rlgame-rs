use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
pub struct Spell {
    pub name: String,
    pub effects: HashMap<String, String>,
    pub mana_cost: i32,
}
