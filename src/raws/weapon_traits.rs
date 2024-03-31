use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct WeaponTrait {
    pub name: String,
    pub effects: HashMap<String, String>,
}
