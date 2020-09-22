use crossterm::{style::Color, Result};

use rand::Rng;
use rc_game::{
    Game, GameState, Player, Position, Renderable, RogueCrossGame, GAME_COLS, GAME_ROWS,
};
use specs::{Builder, World, WorldExt};

#[derive(PartialEq, Copy, Clone)]
enum TileType {
    Wall,
    Floor,
}

fn xy_idx(x: u16, y: u16) -> usize {
    (y as usize * GAME_COLS as usize) + x as usize
}

fn idx_xy(idx: usize) -> (u16, u16) {
    let x = idx % GAME_COLS as usize;
    let y = idx / GAME_COLS as usize;
    (x as u16, y as u16)
}

fn new_map() -> Vec<TileType> {
    let mut map = vec![TileType::Floor; (GAME_COLS * GAME_ROWS) as usize];
    // Walls

    for x in 0..GAME_COLS {
        map[xy_idx(x, 0)] = TileType::Wall;
        map[xy_idx(x, GAME_ROWS - 1)] = TileType::Wall;
    }
    for y in 0..GAME_ROWS {
        map[xy_idx(0, y)] = TileType::Wall;
        map[xy_idx(GAME_COLS - 1, y)] = TileType::Wall;
    }

    let mut rng = rand::thread_rng();
    for _ in 0..400 {
        let x = rng.gen_range(1, GAME_COLS - 1);
        let y = rng.gen_range(1, GAME_ROWS - 1);

        // TODO: expose player start coords
        if x != 40 || y != 25 {
            let idx = xy_idx(x, y);
            map[idx] = TileType::Wall;
        }
    }

    map
}

#[derive(Default)]
struct Ch03Game {}

impl Game for Ch03Game {
    fn init(&self, _: &GameState, ecs: &mut World) -> Result<()> {
        let map = new_map();
        for idx in 0..map.len() {
            let tile = map.get(idx);
            let (x, y) = idx_xy(idx);
            if tile.is_none() {
                continue;
            }
            let renderable = match tile.unwrap() {
                TileType::Wall => Renderable {
                    glyph: '#',
                    fg: Color::DarkGrey,
                    bg: None,
                },
                TileType::Floor => Renderable {
                    glyph: '.',
                    fg: Color::Yellow,
                    bg: None,
                },
            };
            ecs.create_entity()
                .with(Position {
                    x: x as i32,
                    y: y as i32,
                })
                .with(renderable)
                .build();
        }

        ecs.create_entity()
            .with(Position { x: 40, y: 12 })
            .with(Renderable {
                glyph: '@',
                fg: Color::Yellow,
                bg: None,
            })
            .with(Player {})
            .build();

        Ok(())
    }

    fn update(&mut self, _: &GameState, _: &World) -> Result<()> {
        Ok(())
    }

    fn render(&self, _: &GameState, _: &mut std::io::Stdout) -> Result<()> {
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut game: RogueCrossGame<Ch03Game> = Default::default();
    game.start()
}
