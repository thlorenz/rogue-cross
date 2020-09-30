mod components;
mod enums;
mod offset;
mod rc_terminal;
mod renderables;
mod renderer;
use crate::rc_terminal::*;
pub use components::*;
pub use enums::*;
pub use offset::Offset;
use renderables::{renderable_floor, renderable_wall};

use crossterm::{
    cursor, event::poll, event::read, event::Event, event::KeyCode, event::KeyEvent,
    event::KeyModifiers, execute, style::Color, terminal, Result,
};

use renderer::Renderer;
use specs::prelude::*;
use terminal::{disable_raw_mode, enable_raw_mode, ClearType};

use std::{io::stdout, io::Stdout, io::Write, thread::sleep, time::Duration, time::SystemTime};

const FRAMES_PER_SEC: u64 = 60;
const MS_PER_FRAME: u64 = 1_000 / FRAMES_PER_SEC;

pub const GAME_COLS: u16 = 80;
pub const GAME_ROWS: u16 = 25;

pub fn create_blank_map(gs: &GameState) -> Vec<TileType> {
    vec![TileType::Empty; (gs.rows * gs.cols) as usize]
}

#[allow(unused)]
pub trait Game: 'static + Default {
    fn init(&self, gs: &GameState, ecs: &mut World) -> Result<()> {
        Ok(())
    }
    fn update(&mut self, gs: &GameState, ecs: &World) -> Result<()> {
        Ok(())
    }
}

pub struct GameState {
    pub cols: u16,
    pub rows: u16,
    event: Option<Event>,
}

pub struct RogueCrossGame<TGame>
where
    TGame: Game,
{
    ecs: World,
    map: Vec<TileType>,
    game: TGame,
    game_state: GameState,
    millis_per_frame: u64,
    renderer: Option<Renderer>,
    should_exit: bool,
    stdout: Stdout,
    title: String,
    player_start_position: Offset,
    built_map: bool,
    started: bool,
}

fn centered_origin(cols: u16, rows: u16) -> Result<Offset> {
    let (w, h) = terminal::size()?;
    let margin_x = if w > cols { (w - cols) / 2 } else { 0 };

    let margin_y = if h > rows { (h - rows) / 2 } else { 0 };

    // Place origin inside terminal frame
    Ok(Offset::new(margin_x + 1, margin_y + 1))
}

impl<TGame> Default for RogueCrossGame<TGame>
where
    TGame: Game,
{
    fn default() -> Self {
        let mut ecs = World::new();
        ecs.register::<Position>();
        ecs.register::<Renderable>();
        ecs.register::<Collider>();
        ecs.register::<Player>();
        let game_state = GameState {
            cols: 80,
            rows: 25,
            event: None,
        };
        let stdout: Stdout = stdout();
        let map = create_blank_map(&game_state);
        let player_start_position = Offset { x: 40, y: 12 };

        Self {
            ecs,
            map,
            game: Default::default(),
            game_state,
            player_start_position,
            millis_per_frame: MS_PER_FRAME,
            renderer: None,
            should_exit: false,
            stdout,
            title: "Rogue Cross Game".to_string(),
            built_map: false,
            started: false,
        }
    }
}

impl<TGame> RogueCrossGame<TGame>
where
    TGame: Game,
{
    pub fn build_map(
        &mut self,
        create_map: fn(gs: &GameState, player_position: &Offset) -> Vec<TileType>,
    ) {
        assert!(!self.started, "Need to build map before starting the game");
        self.map = create_map(&self.game_state, &self.player_start_position).to_vec();
        self.built_map = true;
    }

    pub fn set_player_start(&mut self, pos: Offset) {
        assert!(
            !self.started,
            "Need to set player position before starting the game"
        );
        assert!(
            !self.built_map,
            "Need to set player position before building the map"
        );
        self.player_start_position = pos;
    }

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
        self.init_map_entities();
        self.init_player();

        enable_raw_mode()?;

        execute!(
            self.stdout,
            terminal::SetTitle(&self.title),
            terminal::Clear(ClearType::All),
            cursor::Hide,
        )?;

        let cols = self.game_state.cols;
        let rows = self.game_state.rows;
        let origin = centered_origin(cols, rows)?;

        draw_terminal_frame(&mut self.stdout, &origin, cols as u16, rows as u16)?;

        self.renderer = Some(Renderer::new(origin, cols, rows));

        self.game.init(&self.game_state, &mut self.ecs)?;

        self.ecs.maintain();
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

    fn idx_xy(&self, idx: usize) -> Offset {
        let x = idx as u16 % self.game_state.cols;
        let y = idx as u16 / self.game_state.cols;
        Offset::new(x, y)
    }

    fn xy_idx(&self, x: usize, y: usize) -> usize {
        (y * self.game_state.cols as usize) + x
    }

    fn init_map_entities(&mut self) {
        for (idx, tile) in self.map.iter().enumerate() {
            let Offset { x, y } = self.idx_xy(idx);
            let renderable = match tile {
                TileType::Floor => renderable_floor(),
                TileType::Wall => renderable_wall(),
                TileType::Empty => Renderable::default(),
            };
            self.ecs
                .create_entity()
                .with(Position { x, y })
                .with(renderable)
                .build();
        }
    }

    fn init_player(&mut self) {
        self.ecs
            .create_entity()
            .with::<Position>((&self.player_start_position).into())
            .with(Renderable {
                glyph: '@',
                fg: Color::Yellow,
                bg: None,
            })
            .with(Player {})
            .build();
    }

    //
    // Rendering
    //
    fn render(&mut self) -> Result<()> {
        let out = &mut self.stdout;
        let renderer = self.renderer.as_mut().unwrap();

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
            renderer.render(pos.x, pos.y, render);
        }

        renderer.flush(out)
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
        let x = pos.x + dx;
        let y = pos.y + dy;
        let idx = self.xy_idx(x as usize, y as usize);
        let blocked = match self.map[idx] {
            TileType::Wall => true,
            TileType::Empty | TileType::Floor => false,
        };
        if !blocked {
            pos.x = x;
            pos.y = y;
            self.clamp_position(pos)
        }
    }

    fn move_player(&self, dx: i32, dy: i32) {
        let mut positions = self.ecs.write_storage::<Position>();
        let players = self.ecs.read_storage::<Player>();

        let player_positions = (&players, &mut positions).join();

        for (_, pos) in player_positions {
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

