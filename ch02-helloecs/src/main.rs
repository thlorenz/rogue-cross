use crossterm::{style::Color, Result};

use rc_game::{Game, Player, Position, Renderable, RogueCrossGame, GAME_COLS, GAME_ROWS};
use specs::{prelude::*, Builder, World, WorldExt};
use specs_derive::*;

#[derive(Component)]
struct LeftMover {
    pub min_col: i32,
    pub max_col: i32,
    pub min_row: i32,
    pub max_row: i32,
}

impl Default for LeftMover {
    fn default() -> Self {
        Self {
            min_col: 0,
            max_col: GAME_COLS as i32,
            min_row: 0,
            max_row: GAME_ROWS as i32,
        }
    }
}

struct LeftWalker {}

impl<'a> System<'a> for LeftWalker {
    type SystemData = (ReadStorage<'a, LeftMover>, WriteStorage<'a, Position>);

    fn run(&mut self, (lefty, mut pos): Self::SystemData) {
        for (lefty, pos) in (&lefty, &mut pos).join() {
            pos.x -= 1;
            if pos.x < 0 {
                pos.x = lefty.max_col;
            }
        }
    }
}

#[derive(Default)]
struct Ch02Game {}
impl Game for Ch02Game {
    fn init(&self, ecs: &mut World) -> Result<()> {
        ecs.register::<Renderable>();
        ecs.register::<LeftMover>();

        ecs.create_entity()
            .with(Position { x: 40, y: 12 })
            .with(Renderable {
                glyph: '@',
                fg: Color::Yellow,
                bg: None,
            })
            .with(Player {})
            .build();

        for i in 0..10 {
            ecs.create_entity()
                .with(Position { x: i * 7, y: 10 })
                .with(Renderable {
                    glyph: '☺',
                    fg: Color::Red,
                    bg: None,
                })
                .with(LeftMover::default())
                .build();
        }
        Ok(())
    }

    fn update(&mut self, ecs: &World) -> Result<()> {
        let mut lw = LeftWalker {};
        lw.run_now(ecs);
        Ok(())
    }

    fn render(&self, _: &mut std::io::Stdout) -> Result<()> {
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut game: RogueCrossGame<Ch02Game> = Default::default();
    game.start()
}
