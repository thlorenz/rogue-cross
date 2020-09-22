mod components;
mod offset;
mod rc_terminal;
use crate::rc_terminal::*;
pub use components::*;
use offset::Offset;

use crossterm::{
    cursor, event::poll, event::read, event::Event, event::KeyCode, event::KeyEvent,
    event::KeyModifiers, execute, queue, style::Print, style::ResetColor,
    style::SetBackgroundColor, style::SetForegroundColor, terminal, Result,
};

use specs::prelude::*;
use terminal::{disable_raw_mode, enable_raw_mode, ClearType};

use std::{io::stdout, io::Stdout, io::Write, thread::sleep, time::Duration, time::SystemTime};

const FRAMES_PER_SEC: u64 = 10;
const MS_PER_FRAME: u64 = 1_000 / FRAMES_PER_SEC;

pub const GAME_COLS: u16 = 80;
pub const GAME_ROWS: u16 = 25;

pub trait Game: 'static + Default {
    fn init(&self, gs: &GameState, ecs: &mut World) -> Result<()>;
    fn update(&mut self, gs: &GameState, ecs: &World) -> Result<()>;
    fn render(&self, gs: &GameState, out: &mut Stdout) -> Result<()>;
}

pub struct GameState {
    cols: u16,
    rows: u16,
    event: Option<Event>,
}

pub struct RogueCrossGame<TGame>
where
    TGame: Game,
{
    title: String,
    game_state: GameState,
    ecs: World,
    stdout: Stdout,
    origin: Offset,
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
        ecs.register::<Renderable>();
        ecs.register::<Player>();
        let game_state = GameState {
            cols: 80,
            rows: 25,
            event: None,
        };
        let stdout: Stdout = stdout();
        let origin = Offset::new(1, 1);
        Self {
            title: "Rogue Cross Game".to_string(),
            game_state,
            ecs,
            millis_per_frame: MS_PER_FRAME,
            stdout,
            origin,
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
            // execute!(self.stdout, Print(format!("{}\n", self.origin)))?;
            self.center()?;
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

        self.center()?;
        self.game.init(&self.game_state, &mut self.ecs)?;

        draw_terminal_frame(
            &mut self.stdout,
            &self.origin,
            self.game_state.cols as u16,
            self.game_state.rows as u16,
        )?;
        self.stdout.flush()?;
        Ok(())
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
        self.game_state.event = None;
        if poll(Duration::from_millis(self.millis_per_frame))? {
            self.game_state.event = Some(read()?);
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
    fn center(&mut self) -> Result<()> {
        let (w, h) = terminal::size()?;
        let margin_x = if w > self.game_state.cols {
            (w - self.game_state.cols) / 2
        } else {
            0
        };

        let margin_y = if h > self.game_state.rows {
            (h - self.game_state.rows) / 2
        } else {
            0
        };

        // Place origin inside terminal frame
        self.origin = Offset::new(margin_x + 1, margin_y + 1);
        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        let out = &mut self.stdout;

        cls(
            out,
            &self.origin,
            self.game_state.cols as u16,
            self.game_state.rows as u16,
        )?;

        let positions = self.ecs.read_storage::<Position>();
        let renderables = self.ecs.read_storage::<Renderable>();

        for (pos, render) in (&positions, &renderables).join() {
            if 0 > pos.x
                || pos.x >= self.game_state.cols as i32
                || 0 > pos.y
                || pos.y >= self.game_state.rows as i32
            {
                continue;
            }
            match render.bg {
                None => queue!(out, ResetColor),
                Some(color) => queue!(out, SetBackgroundColor(color)),
            }?;
            // TODO: consider optimization options:
            // 1. no cls if all squares are redrawn
            // 2. keep internal buffer of what we drew to the screen and only draw it again if
            //    it is different now
            // 3. switch outputs?

            // NOTE: not a huge fan of having to adjust pos by 1 since this also has to be
            // applied inside the Game::render methods.
            // Not changing this yet as it might turn out that all the rendering is done inside
            // this lib.
            let render_pos = Offset::from(pos).translate(&self.origin);
            queue!(
                out,
                cursor::MoveTo(render_pos.x as u16, render_pos.y as u16),
                SetForegroundColor(render.fg),
                Print(render.glyph)
            )?;
        }

        self.game.render(&self.game_state, out)?;

        out.flush()?;
        Ok(())
    }

    //
    // Updates
    //
    fn update(&mut self) -> Result<()> {
        self.process_input();
        self.game.update(&self.game_state, &mut self.ecs)?;
        Ok(())
    }

    fn process_input(&mut self) {
        match self.game_state.event {
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
        let maxx = self.game_state.cols as i32 - 1;
        let miny = 0;
        let maxy = self.game_state.rows as i32 - 1;
        pos.clamp(minx, maxx, miny, maxy)
    }
}
