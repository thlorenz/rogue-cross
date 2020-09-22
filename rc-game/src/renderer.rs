use crate::{offset::Offset, Renderable};
use crossterm::{
    cursor, queue, style::Print, style::ResetColor, style::SetBackgroundColor,
    style::SetForegroundColor, Result,
};
use std::io::Write;

pub struct Renderer {
    previous_buffer: Vec<Renderable>,
    current_buffer: Vec<Renderable>,
    origin: Offset,
    cols: u16,
    buffer_size: usize,
}

impl Renderer {
    pub fn new(origin: Offset, cols: u16, rows: u16) -> Self {
        let buffer_size = (cols * rows) as usize;
        let previous_buffer = vec![Renderable::default(); buffer_size];
        let current_buffer = vec![Renderable::default(); buffer_size];

        Self {
            previous_buffer,
            current_buffer,
            origin,
            cols,
            buffer_size,
        }
    }

    pub fn render(&mut self, x: i32, y: i32, renderable: &Renderable) {
        let idx = self.xy_idx(x, y);
        self.current_buffer[idx] = renderable.clone()
    }

    pub fn flush<W>(&mut self, w: &mut W) -> Result<()>
    where
        W: Write,
    {
        for idx in 0..self.buffer_size {
            if self.previous_buffer[idx] == self.current_buffer[idx] {
                continue;
            }
            let pos = self.origin.translate(&self.idx_xy(idx));
            let render = &self.current_buffer[idx];

            match render.bg {
                None => queue!(w, ResetColor),
                Some(color) => queue!(w, SetBackgroundColor(color)),
            }?;
            queue!(
                w,
                cursor::MoveTo(pos.x as u16, pos.y as u16),
                SetForegroundColor(render.fg),
                Print(render.glyph)
            )?;
            self.previous_buffer[idx] = render.clone();
        }

        w.flush()?;
        Ok(())
    }

    fn xy_idx<T: Into<i32>>(&self, x: T, y: T) -> usize {
        (y.into() as usize * self.cols as usize) + x.into() as usize
    }

    fn idx_xy(&self, idx: usize) -> Offset {
        let x = idx as u16 % self.cols;
        let y = idx as u16 / self.cols;
        Offset::new(x, y)
    }
}
