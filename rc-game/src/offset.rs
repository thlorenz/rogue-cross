use std::fmt::Display;

use crate::components::Position;

#[derive(Default)]
pub struct Offset {
    pub x: i32,
    pub y: i32,
}

impl Offset {
    pub fn new<T>(dx: T, dy: T) -> Self
    where
        T: Into<i32>,
    {
        Self {
            x: dx.into(),
            y: dy.into(),
        }
    }

    pub fn apply<T>(&self, x: T, y: T) -> (i32, i32)
    where
        T: Into<i32>,
    {
        (self.x + x.into(), self.y + y.into())
    }

    pub fn _translate_xy<T>(&self, dx: T, dy: T) -> Offset
    where
        T: Into<i32>,
    {
        Offset {
            x: self.x + dx.into(),
            y: self.y + dy.into(),
        }
    }

    pub fn translate(&self, offset: &Offset) -> Offset {
        Offset {
            x: self.x + offset.x,
            y: self.y + offset.y,
        }
    }
}

impl Display for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Offset({}, {})", self.x, self.y))
    }
}

impl From<&Position> for Offset {
    fn from(pos: &Position) -> Self {
        Self::new(pos.x, pos.y)
    }
}
