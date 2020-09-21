mod rc_terminal;
use self::rc_terminal::*;

use crossterm::{
    cursor, event::poll, event::read, event::Event, event::KeyCode, event::KeyEvent,
    event::KeyModifiers, execute, queue, style::Color, style::Print, style::ResetColor,
    style::SetBackgroundColor, style::SetForegroundColor, terminal, Result,
};

use specs::prelude::*;
use specs_derive::*;
use terminal::{disable_raw_mode, enable_raw_mode, ClearType};

use std::{
    cmp::{max, min},
    io::stdout,
    io::Stdout,
    io::Write,
    thread::sleep,
    time::Duration,
    time::SystemTime,
};

#[derive(Component)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    fn clamp(&mut self, minx: i32, maxx: i32, miny: i32, maxy: i32) {
        self.x = min(maxx, max(minx, self.x));
        self.y = min(maxy, max(miny, self.y));
    }
}

#[derive(Component, Debug)]
pub struct Player {}

#[derive(Component)]
pub struct Renderable {
    pub glyph: char,
    pub fg: Color,
    pub bg: Option<Color>,
}

const FRAMES_PER_SEC: u64 = 60;
const MS_PER_FRAME: u64 = 1_000 / FRAMES_PER_SEC;

pub const GAME_COLS: u16 = 80;
pub const GAME_ROWS: u16 = 25;

pub trait Game: 'static + Default {
    fn init(&self, ecs: &mut World) -> Result<()>;
    fn update(&mut self, ecs: &World) -> Result<()>;
    fn render(&self, _: &mut Stdout) -> Result<()>;
}

pub struct RogueCrossGame<TGame>
where
    TGame: Game,
{
    cols: u16,
    rows: u16,
    title: String,
    ecs: World,
    event: Option<Event>,
    stdout: Stdout,
    millis_per_frame: u64,
    should_exit: bool,
    game: TGame,
}

impl<TGame> Default for RogueCrossGame<TGame>
where
    TGame: Game,
{
    fn default() -> Self {
        let mut ecs = World::new();
        ecs.register::<Position>();
        ecs.register::<Player>();
        Self {
            cols: 80,
            rows: 25,
            title: "Rogue Cross Game".to_string(),
            ecs,
            event: None,
            stdout: stdout(),
            millis_per_frame: MS_PER_FRAME,
            should_exit: false,
            game: Default::default(),
        }
    }
}

impl<TGame> RogueCrossGame<TGame>
where
    TGame: Game,
{
    pub fn start(&mut self) -> Result<()> {
        self.init()?;

        loop {
            let loop_start = SystemTime::now();

            self.poll()?;
            self.update()?;
            if self.should_exit {
                break;
            }
            self.render()?;

            self.enforce_framerate(&loop_start);
        }

        self.deinit()
    }

    fn init(&mut self) -> Result<()> {
        enable_raw_mode()?;

        execute!(
            self.stdout,
            terminal::SetTitle(&self.title),
            terminal::Clear(ClearType::All),
            cursor::Hide,
        )?;
        self.game.init(&mut self.ecs)?;
        draw_terminal_frame(&mut self.stdout, self.cols as u16, self.rows as u16)
    }

    fn deinit(&mut self) -> Result<()> {
        execute!(
            self.stdout,
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0),
            cursor::Show,
        )?;
        disable_raw_mode()
    }

    fn poll(&mut self) -> Result<()> {
        self.event = None;
        if poll(Duration::from_millis(self.millis_per_frame))? {
            self.event = Some(read()?);
        }
        Ok(())
    }

    fn enforce_framerate(&self, loop_start: &SystemTime) {
        // Ensure consistent framerate even if we get user input at the top of the frame
        let elapsed = loop_start.elapsed().expect("Could not get loop start time");
        let ms = elapsed.as_millis() as u64;
        if ms < self.millis_per_frame {
            let remaining = self.millis_per_frame - ms;
            sleep(Duration::from_millis(remaining))
        }
    }

    //
    // Rendering
    //
    fn render(&mut self) -> Result<()> {
        let out = &mut self.stdout;

        cls(out, self.cols as u16, self.rows as u16)?;

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            if 0 > pos.x || pos.x >= self.cols as i32 || 0 > pos.y || pos.y >= self.rows as i32 {
                continue;
            }
            match render.bg {
                None => queue!(out, ResetColor),
                Some(color) => queue!(out, SetBackgroundColor(color)),
            }?;
            // Offset coords by 1 to account for terminal frame
            queue!(
                out,
                cursor::MoveTo(pos.x as u16 + 1, pos.y as u16 + 1),
                SetForegroundColor(render.fg),
                Print(render.glyph)
            )?;
        }

        self.game.render(out)?;

        out.flush()?;
        Ok(())
    }

    //
    // Updates
    //
    fn update(&mut self) -> Result<()> {
        self.process_input();
        self.game.update(&mut self.ecs)?;
        Ok(())
    }

    fn process_input(&mut self) {
        match self.event {
            Some(Event::Key(KeyEvent {
                modifiers: KeyModifiers::NONE,
                code,
            })) => match code {
                KeyCode::Left | KeyCode::Char('a') => self.move_player(-1, 0),
                KeyCode::Right | KeyCode::Char('d') => self.move_player(1, 0),
                KeyCode::Up | KeyCode::Char('w') => self.move_player(0, -1),
                KeyCode::Down | KeyCode::Char('s') => self.move_player(0, 1),
                KeyCode::Esc => self.should_exit = true,
                _ => {}
            },
            _ => {}
        }
    }

    fn move_by(&self, pos: &mut Position, dx: i32, dy: i32) {
        pos.x += dx;
        pos.y += dy;
        self.clamp_position(pos)
    }

    fn move_player(&self, dx: i32, dy: i32) {
        let mut positions = self.ecs.write_storage::<Position>();
        let mut players = self.ecs.write_storage::<Player>();

        for (_, pos) in (&mut players, &mut positions).join() {
            self.move_by(pos, dx, dy)
        }
    }

    fn clamp_position(&self, pos: &mut Position) {
        let minx = 0;
        let maxx = self.cols as i32 - 2;
        let miny = 0;
        let maxy = self.rows as i32 - 2;
        pos.clamp(minx, maxx, miny, maxy)
    }
}
