use super::Position;

#[derive(serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct Rect {
    pub x1: i32,
    pub x2: i32,
    pub y1: i32,
    pub y2: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> Rect {
        Rect {
            x1: x,
            x2: x + w,
            y1: y,
            y2: y + h,
        }
    }

    /// Returns true if rectangle intersets with other.
    pub fn intersects(&self, other: &Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    /// Returns center position of a Rectangle
    pub fn center(&self) -> Position {
        Position {
            x: (self.x1 + self.x2) / 2,
            y: (self.y1 + self.y2) / 2,
        }
    }
}
