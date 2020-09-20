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
    time::Duration,
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
    bg: Color,
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
        pos.x = min(COLS - 1, max(0, pos.x + dx));
        pos.y = min(ROWS - 1, max(0, pos.y + dy));
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
        queue!(
            w,
            ResetColor,
            terminal::Clear(ClearType::All),
            terminal::SetTitle("Rougelike Tutorial"),
            cursor::Hide,
        )?;

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            if 0 > pos.x || pos.x >= COLS || 0 > pos.y || pos.y >= ROWS {
                continue;
            }
            queue!(
                w,
                cursor::MoveTo(pos.x as u16, pos.y as u16),
                // TODO: make bg an option and None to mean terminal background
                ResetColor,
                // SetBackgroundColor(render.bg),
                SetForegroundColor(render.fg),
                Print(render.glyph),
            )?;
        }

        w.flush()?;
        Ok(())
    }
}

fn main_loop<W>(w: &mut W, gs: &mut State) -> Result<()>
where
    W: Write,
{
    loop {
        // TODO: draw terminal frame
        // TODO: draw frame rate just outside upper right corner.
        gs.event = None;
        if poll(Duration::from_millis(100))? {
            gs.event = Some(read()?);
            // TODO: Make sure that if we get input early that we wait the extra time to get
            // a constant frame rate.
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
            bg: Color::Black,
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
                bg: Color::Black,
            })
            .with(LeftMover {})
            .build();
    }

    main_loop(&mut stdout, &mut gs)?;

    disable_raw_mode()
}
