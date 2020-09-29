use crossterm::Result;

use rand::Rng;
use rc_game::{Game, GameState, Offset, RogueCrossGame, TileType};

fn xy_idx(x: usize, y: usize, cols: usize) -> usize {
    (y * cols) + x
}

#[derive(Default)]
struct Ch03Game {}

impl Game for Ch03Game {}

fn create_map(gs: &GameState, player_position: &Offset) -> Vec<TileType> {
    let mut map = vec![TileType::Empty; (gs.rows * gs.cols) as usize];
    let GameState { cols, rows, .. } = *gs;
    let cols = cols as usize;
    let rows = rows as usize;
    // Walls
    for x in 0..cols {
        map[xy_idx(x, 0, cols)] = TileType::Wall;
        map[xy_idx(x, rows - 1, cols)] = TileType::Wall;
    }
    for y in 0..rows {
        map[xy_idx(0, y, cols)] = TileType::Wall;
        map[xy_idx(cols - 1, y, cols)] = TileType::Wall;
    }

    let mut rng = rand::thread_rng();
    for _ in 0..400 {
        let x = rng.gen_range(1, cols - 1);
        let y = rng.gen_range(1, rows - 1);

        if x != player_position.x as usize || y != player_position.y as usize {
            let idx = xy_idx(x, y, cols);
            map[idx] = TileType::Wall;
        }
    }
    map
}

fn main() -> Result<()> {
    let mut game: RogueCrossGame<Ch03Game> = Default::default();
    game.set_player_start(Offset { x: 5, y: 12 });
    game.build_map(create_map);
    game.start()
}
