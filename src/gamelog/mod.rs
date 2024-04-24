use rltk::RGB;

mod builder;
pub mod events;
mod logstore;
pub use builder::*;
#[cfg(not(target_arch = "wasm32"))]
pub use logstore::clone_log;
pub use logstore::{clear_log, print_log, restore_log};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LogFragment {
    pub color: RGB,
    pub text: String,
}
