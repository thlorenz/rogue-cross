use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    execute, queue,
    style::{Print, ResetColor},
    terminal::{self, disable_raw_mode, enable_raw_mode, ClearType},
    Result,
};
use std::{
    io::{stdout, Write},
    time::Duration,
};

fn game_loop<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    loop {
        queue!(
            w,
            ResetColor,
            terminal::Clear(ClearType::All),
            terminal::SetTitle("Rougelike Tutorial"),
            cursor::Hide,
            cursor::MoveTo(1, 1),
            Print("Hello Rust World")
        )?;
        w.flush()?;

        if poll(Duration::from_millis(200))? {
            let event = read()?;
            if event == Event::Key(KeyCode::Esc.into()) {
                break;
            }
        }
    }

    execute!(w, ResetColor, cursor::Show)?;
    Ok(())
}

fn main() -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = stdout();
    game_loop(&mut stdout)?;

    disable_raw_mode()
}
