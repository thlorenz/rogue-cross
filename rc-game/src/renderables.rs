use crossterm::style::Color;

use crate::Renderable;

pub fn renderable_floor() -> Renderable {
    Renderable {
        glyph: '.',
        fg: Color::Yellow,
        bg: None,
    }
}

pub fn renderable_wall() -> Renderable {
    Renderable {
        glyph: '#',
        fg: Color::DarkGrey,
        bg: None,
    }
}
