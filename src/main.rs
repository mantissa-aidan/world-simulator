use ggez::{event, GameResult};
use ggez::{ContextBuilder};
use ggez::conf::{WindowMode, WindowSetup};

use world_simulator::{
    Simulation,
    constants::{WINDOW_WIDTH, WINDOW_HEIGHT, WORLD_WIDTH, WORLD_HEIGHT},
};

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("world_simulator", "Aidan")
        .window_mode(WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .window_setup(WindowSetup::default().title("World Simulator"))
        .build()?;

    let simulation = Simulation::new(WORLD_WIDTH, WORLD_HEIGHT);
    event::run(ctx, event_loop, simulation)
}
