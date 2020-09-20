use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::Color,
    style::Print,
    style::ResetColor,
    style::SetBackgroundColor,
    style::SetForegroundColor,
    terminal::{self, disable_raw_mode, enable_raw_mode, ClearType},
    Result,
};

const FRAMES_PER_SEC: u64 = 60;
const MS_PER_FRAME: u64 = 1_000 / FRAMES_PER_SEC;

// Explicitly importing each spec part separately, even from the prelude to better understand
// where they come from.
// In future chapters we'll just import prelude::*.
use specs::{
    prelude::{Component, DenseVecStorage, Join, ReadStorage, System, WriteStorage},
    Builder, RunNow, World, WorldExt,
};
use specs_derive::*;
use std::{
    cmp::max,
    cmp::min,
    io::{stdout, Write},
    thread::sleep,
    time::Duration,
    time::SystemTime,
};

pub trait GameState: 'static {
    fn tick<W>(&mut self, w: &mut W) -> Result<()>
    where
        W: Write;
}

#[derive(Component)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component)]
struct Renderable {
    glyph: char,
    fg: Color,
    bg: Option<Color>,
}

#[derive(Component)]
struct LeftMover {}

#[derive(Component, Debug)]
struct Player {}

struct LeftWalker {}

impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        for (_lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 {
                pos.x = 79;
            }
        }
    }
}

const ROWS: i32 = 50;
const COLS: i32 = 80;

fn try_move_player(dx: i32, dy: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();

    for (_, pos) in (&mut players, &mut positions).join() {
        pos.x = min(COLS - 2, max(0, pos.x + dx));
        pos.y = min(ROWS - 2, max(0, pos.y + dy));
    }
}

fn player_input(gs: &mut State) {
    match gs.event {
        Some(Event::Key(KeyEvent {
            modifiers: KeyModifiers::NONE,
            code,
        })) => match code {
            KeyCode::Left => try_move_player(-1, 0, &mut gs.ecs),
            KeyCode::Right => try_move_player(1, 0, &mut gs.ecs),
            KeyCode::Up => try_move_player(0, -1, &mut gs.ecs),
            KeyCode::Down => try_move_player(0, 1, &mut gs.ecs),
            KeyCode::Esc => gs.should_exit = true,
            _ => {}
        },
        _ => {}
    }
}

struct State {
    ecs: World,
    should_exit: bool,
    event: Option<Event>,
}

impl State {
    fn run_systems(&mut self) {
        let mut lw = LeftWalker {};
        lw.run_now(&self.ecs);
        self.ecs.maintain();
    }
}

impl GameState for State {
    fn tick<W>(&mut self, w: &mut W) -> Result<()>
    where
        W: Write,
    {
        // Update
        player_input(self);
        self.run_systems();

        // Render
        cls(w, COLS as u16, ROWS as u16)?;

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            if 0 > pos.x || pos.x >= COLS || 0 > pos.y || pos.y >= ROWS {
                continue;
            }
            match render.bg {
                None => queue!(w, ResetColor),
                Some(color) => queue!(w, SetBackgroundColor(color)),
            }?;
            // Offset coords by 1 since on terminal frame is at 0
            queue!(
                w,
                cursor::MoveTo(pos.x as u16 + 1, pos.y as u16 + 1),
                SetForegroundColor(render.fg),
                Print(render.glyph)
            )?;
        }

        w.flush()?;
        Ok(())
    }
}

const UPPER_LEFT_CORNER: char = '╔';
const UPPER_RIGHT_CORNER: char = '╗';
const LOWER_LEFT_CORNER: char = '╚';
const LOWER_RIGHT_CORNER: char = '╝';
const VERTICAL_WALL: char = '║';
const HORIZONTAL_WALL: char = '═';

fn draw_terminal_frame<W>(w: &mut W, ncols: u16, nrows: u16) -> Result<()>
where
    W: Write,
{
    // Corners
    queue!(
        w,
        cursor::MoveTo(0, 0),
        Print(UPPER_LEFT_CORNER),
        cursor::MoveTo(ncols, 0),
        Print(UPPER_RIGHT_CORNER),
        cursor::MoveTo(ncols, nrows),
        Print(LOWER_RIGHT_CORNER),
        cursor::MoveTo(0, nrows),
        Print(LOWER_LEFT_CORNER),
    )?;

    for col in 1..ncols {
        queue!(
            w,
            cursor::MoveTo(col, 0),
            Print(HORIZONTAL_WALL),
            cursor::MoveTo(col, nrows),
            Print(HORIZONTAL_WALL)
        )?
    }
    for row in 1..nrows {
        queue!(
            w,
            cursor::MoveTo(0, row),
            Print(VERTICAL_WALL),
            cursor::MoveTo(ncols, row),
            Print(VERTICAL_WALL)
        )?
    }
    Ok(())
}

/// Clear everything except the terminal frame to minimize flicker
fn cls<W>(w: &mut W, ncols: u16, nrows: u16) -> Result<()>
where
    W: Write,
{
    queue!(w, ResetColor)?;
    for row in 1..nrows {
        queue!(
            w,
            cursor::MoveTo(1, row),
            terminal::Clear(ClearType::UntilNewLine),
            // We cannot avoid clearing the right most column, so we just redraw that afterwards
            cursor::MoveTo(ncols, row),
            Print(VERTICAL_WALL)
        )?
    }
    Ok(())
}

fn main_loop<W>(w: &mut W, gs: &mut State) -> Result<()>
where
    W: Write,
{
    loop {
        // TODO: draw frame rate just outside upper right corner.
        gs.event = None;
        let now = SystemTime::now();
        if poll(Duration::from_millis(MS_PER_FRAME))? {
            gs.event = Some(read()?);

            // Ensure consistent framerate even if we get user input at the top of the frame
            let elapsed = now.elapsed().expect("Could not get elapsed time");
            let ms = elapsed.as_millis() as u64;
            if ms < MS_PER_FRAME {
                let remaining = MS_PER_FRAME - ms;
                sleep(Duration::from_millis(remaining))
            }
        }

        gs.tick(w)?;

        if gs.should_exit {
            break;
        }
    }

    execute!(w, ResetColor, cursor::Show)?;
    Ok(())
}

fn main() -> Result<()> {
    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(
        stdout,
        terminal::SetTitle("Rougelike Tutorial"),
        cursor::Hide,
    )?;
    draw_terminal_frame(&mut stdout, COLS as u16, ROWS as u16)?;

    let mut gs = State {
        ecs: World::new(),
        event: None,
        should_exit: false,
    };

    gs.ecs.register::<Position>();
    gs.ecs.register::<Renderable>();
    gs.ecs.register::<LeftMover>();
    gs.ecs.register::<Player>();

    gs.ecs
        .create_entity()
        .with(Position { x: 40, y: 25 })
        .with(Renderable {
            glyph: '@',
            fg: Color::Yellow,
            bg: None,
        })
        .with(Player {})
        .build();

    for i in 0..10 {
        gs.ecs
            .create_entity()
            .with(Position { x: i * 7, y: 20 })
            .with(Renderable {
                glyph: '☺',
                fg: Color::Red,
                bg: None,
            })
            .with(LeftMover {})
            .build();
    }

    main_loop(&mut stdout, &mut gs)?;

    disable_raw_mode()
}
