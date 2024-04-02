use rltk::RGB;

mod builder;
mod logstore;
pub use builder::*;
pub use logstore::{clear_log, clone_log, log_display, restore_log};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LogFragment {
    pub color: RGB,
    pub text: String,
}
