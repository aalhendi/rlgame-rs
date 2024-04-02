use rltk::RGB;

use super::{logstore::append_entry, LogFragment};

pub struct Logger {
    // current_color: RGB,
    fragments: Vec<LogFragment>,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            // current_color: RGB::named(rltk::WHITE),
            fragments: Vec::new(),
        }
    }

    // pub fn color(mut self, color: (u8, u8, u8)) -> Self {
    //     self.current_color = RGB::named(color);
    //     self
    // }

    // pub fn append<T: Into<String>>(mut self, text: T) -> Self {
    //     self.fragments.push(LogFragment {
    //         color: self.current_color,
    //         text: text.into(),
    //     });
    //     self
    // }

    fn add_fragment(mut self, text: String, color: RGB) -> Self {
        self.fragments.push(LogFragment { color, text });
        self
    }

    pub fn yellow<T: ToString>(self, text: T) -> Self {
        self.add_fragment(text.to_string(), RGB::named(rltk::YELLOW))
    }

    pub fn cyan<T: ToString>(self, text: T) -> Self {
        self.add_fragment(text.to_string(), RGB::named(rltk::CYAN))
    }

    pub fn red<T: std::fmt::Display>(self, text: T) -> Self {
        self.add_fragment(format!("{text}"), RGB::named(rltk::RED))
    }

    pub fn orange<T: std::fmt::Display>(self, text: T) -> Self {
        self.add_fragment(format!("{text}"), RGB::named(rltk::ORANGE))
    }

    pub fn green<T: std::fmt::Display>(self, text: T) -> Self {
        self.add_fragment(format!("{text}"), RGB::named(rltk::GREEN))
    }

    pub fn white<T: Into<String>>(self, text: T) -> Self {
        self.add_fragment(text.into(), RGB::named(rltk::WHITE))
    }

    pub fn magenta<T: Into<String>>(self, text: T) -> Self {
        self.add_fragment(text.into(), RGB::named(rltk::MAGENTA))
    }

    pub fn log(self) {
        append_entry(self.fragments)
    }
}
