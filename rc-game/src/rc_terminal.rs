use crossterm::{cursor, queue, style::Print, style::ResetColor, terminal, Result};

use std::io::Write;

use crate::offset::Offset;

const UPPER_LEFT_CORNER: char = '╔';
const UPPER_RIGHT_CORNER: char = '╗';
const LOWER_LEFT_CORNER: char = '╚';
const LOWER_RIGHT_CORNER: char = '╝';
const VERTICAL_WALL: char = '║';
const HORIZONTAL_WALL: char = '═';

/// terminal frame is drawn around what we consider the terminal
pub fn draw_terminal_frame<W>(w: &mut W, origin: &Offset, ncols: u16, nrows: u16) -> Result<()>
where
    W: Write,
{
    // Offset terminal origin by -1 to draw it around the actual game
    let (minc, minr) = origin.apply(-1, -1);
    let (maxc, maxr) = origin.apply(ncols, nrows);

    // Corners
    queue!(
        w,
        cursor::MoveTo(minc as u16, minr as u16),
        Print(UPPER_LEFT_CORNER),
        cursor::MoveTo(maxc as u16, minr as u16),
        Print(UPPER_RIGHT_CORNER),
        cursor::MoveTo(maxc as u16, maxr as u16),
        Print(LOWER_RIGHT_CORNER),
        cursor::MoveTo(minc as u16, maxr as u16),
        Print(LOWER_LEFT_CORNER),
    )?;

    for col in minc + 1..maxc {
        queue!(
            w,
            cursor::MoveTo(col as u16, minr as u16),
            Print(HORIZONTAL_WALL),
            cursor::MoveTo(col as u16, maxr as u16),
            Print(HORIZONTAL_WALL)
        )?
    }
    for row in minr + 1..maxr {
        queue!(
            w,
            cursor::MoveTo(minc as u16, row as u16),
            Print(VERTICAL_WALL),
            cursor::MoveTo(maxc as u16, row as u16),
            Print(VERTICAL_WALL)
        )?
    }
    Ok(())
}

/// Clear everything except the terminal frame to minimize flicker
pub fn cls<W>(w: &mut W, origin: &Offset, ncols: u16, nrows: u16) -> Result<()>
where
    W: Write,
{
    let (minc, minr) = origin.apply(0, 0);
    let (maxc, maxr) = origin.apply(ncols, nrows);

    queue!(w, ResetColor)?;
    for row in minr..maxr {
        queue!(
            w,
            cursor::MoveTo(minc as u16, row as u16),
            terminal::Clear(terminal::ClearType::UntilNewLine),
            // We cannot avoid clearing the right most column, so we just redraw that afterwards
            cursor::MoveTo(maxc as u16, row as u16),
            Print(VERTICAL_WALL)
        )?
    }
    Ok(())
}
