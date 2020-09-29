use std::cmp::{max, min};

use crossterm::style::Color;
use specs::prelude::*;
use specs_derive::*;

use crate::Offset;

#[derive(Component, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn clamp(&mut self, minx: i32, maxx: i32, miny: i32, maxy: i32) {
        self.x = min(maxx, max(minx, self.x));
        self.y = min(maxy, max(miny, self.y));
    }
}

impl From<&Offset> for Position {
    fn from(offset: &Offset) -> Self {
        let Offset { x, y } = *offset;
        Self { x, y }
    }
}
impl From<Offset> for Position {
    fn from(offset: Offset) -> Self {
        (&offset).into()
    }
}

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component, Clone, PartialEq)]
pub struct Renderable {
    pub glyph: char,
    pub fg: Color,
    pub bg: Option<Color>,
}

impl Default for Renderable {
    fn default() -> Self {
        Self {
            glyph: ' ',
            fg: Color::White,
            bg: None,
        }
    }
}

#[derive(Component, Default)]
pub struct Collider {}
