use crossterm::{cursor, queue, style::Print, style::ResetColor, terminal, Result};

use std::io::Write;

const UPPER_LEFT_CORNER: char = '╔';
const UPPER_RIGHT_CORNER: char = '╗';
const LOWER_LEFT_CORNER: char = '╚';
const LOWER_RIGHT_CORNER: char = '╝';
const VERTICAL_WALL: char = '║';
const HORIZONTAL_WALL: char = '═';

/// terminal frame is drawn around what we consider the terminal
pub fn draw_terminal_frame<W>(w: &mut W, ncols: u16, nrows: u16) -> Result<()>
where
    W: Write,
{
    // Corners
    queue!(
        w,
        cursor::MoveTo(0, 0),
        Print(UPPER_LEFT_CORNER),
        cursor::MoveTo(ncols + 1, 0),
        Print(UPPER_RIGHT_CORNER),
        cursor::MoveTo(ncols + 1, nrows + 1),
        Print(LOWER_RIGHT_CORNER),
        cursor::MoveTo(0, nrows + 1),
        Print(LOWER_LEFT_CORNER),
    )?;

    for col in 1..ncols + 1 {
        queue!(
            w,
            cursor::MoveTo(col, 0),
            Print(HORIZONTAL_WALL),
            cursor::MoveTo(col, nrows + 1),
            Print(HORIZONTAL_WALL)
        )?
    }
    for row in 1..nrows + 1 {
        queue!(
            w,
            cursor::MoveTo(0, row),
            Print(VERTICAL_WALL),
            cursor::MoveTo(ncols + 1, row),
            Print(VERTICAL_WALL)
        )?
    }
    Ok(())
}

/// Clear everything except the terminal frame to minimize flicker
pub fn cls<W>(w: &mut W, ncols: u16, nrows: u16) -> Result<()>
where
    W: Write,
{
    queue!(w, ResetColor)?;
    for row in 1..nrows + 1 {
        queue!(
            w,
            cursor::MoveTo(1, row),
            terminal::Clear(terminal::ClearType::UntilNewLine),
            // We cannot avoid clearing the right most column, so we just redraw that afterwards
            cursor::MoveTo(ncols + 1, row),
            Print(VERTICAL_WALL)
        )?
    }
    Ok(())
}
