mod item_structs;
mod mob_structs;
mod prop_structs;
pub mod rawsmaster;
mod spawn_table_structs;
use serde::Deserialize;
use std::sync::Mutex;
mod loot_structs;
use item_structs::Item;
use loot_structs::LootTable;
use mob_structs::Mob;
use prop_structs::Prop;
use rawsmaster::RawMaster;
use spawn_table_structs::SpawnTableEntry;
pub mod faction_structs;
use faction_structs::FactionInfo;

#[derive(Deserialize, Debug, Default)]
pub struct Raws {
    pub items: Vec<Item>,
    pub mobs: Vec<Mob>,
    pub props: Vec<Prop>,
    pub spawn_table: Vec<SpawnTableEntry>,
    pub loot_tables: Vec<LootTable>,
    pub faction_table: Vec<FactionInfo>,
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
