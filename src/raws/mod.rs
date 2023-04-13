mod item_structs;
mod mob_structs;
mod prop_structs;
pub mod rawsmaster;
use self::{item_structs::Item, mob_structs::Mob, prop_structs::Prop};
use crate::raws::rawsmaster::RawMaster;
use serde::Deserialize;
use std::sync::Mutex;

#[derive(Deserialize, Debug, Default)]
pub struct Raws {
    pub items: Vec<Item>,
    pub mobs: Vec<Mob>,
    pub props: Vec<Prop>,
}

rltk::embedded_resource!(RAW_FILE, "../../raws/spawns.json");

lazy_static! {
    pub static ref RAWS: Mutex<RawMaster> = Mutex::new(RawMaster::default());
}

pub fn load_raws() {
    rltk::link_resource!(RAW_FILE, "../../raws/spawns.json");

    // Retrieve raw data as u8 array
    let raw_data = rltk::embedding::EMBED
        .lock()
        .get_resource("../../raws/spawns.json".to_string())
        .unwrap();
    let raw_string =
        std::str::from_utf8(raw_data).expect("Unable to convert to a valid UTF-8 string.");
    let decoder: Raws = serde_json::from_str(raw_string).expect("Unable to parse JSON");
    RAWS.lock().unwrap().load(decoder);
}
