//! # State System
//! 
//! This module implements the state pattern for managing different simulation states.
//! Each state handles its own update, draw, and input logic.
//! 
//! ## Example
//! ```no_run
//! use world_simulator::states::SimulationState;
//! use world_simulator::world::World;
//! use world_simulator::events::EventSystem;
//! use ggez::error::GameResult;
//! use ggez::event::KeyCode;
//! 
//! struct TestState;
//! 
//! impl SimulationState for TestState {
//!     fn update(&mut self, world: &mut World, _events: &mut EventSystem) -> GameResult {
//!         world.update();
//!         Ok(())
//!     }
//!     
//!     fn draw(&self, world: &World, ctx: &mut ggez::Context) -> GameResult {
//!         world.draw(ctx)
//!     }
//!     
//!     fn handle_input(&mut self, _world: &mut World, _events: &mut EventSystem, 
//!                    _keycode: KeyCode, _ctx: &mut ggez::Context) {
//!         // Handle input here
//!     }
//! }
//! ```

use ggez::graphics::{self, Color, DrawMode, DrawParam, Mesh, Text};
use ggez::{Context, GameResult};
use ggez::event::EventHandler;
use ggez::input::keyboard::KeyCode;
use crate::world::World;
use crate::events::{EventSystem, SimulationEvent};
use ggez::graphics::Canvas;

/// The trait that all simulation states must implement.
/// Each state handles updating the world, drawing to the screen, and processing input.
/// 
/// Example:
/// ```
/// use world_simulator::states::SimulationState;
/// use world_simulator::world::World;
/// use world_simulator::events::EventSystem;
/// use ggez::error::GameResult;
/// use ggez::event::KeyCode;
/// 
/// struct TestState;
/// 
/// impl SimulationState for TestState {
///     fn update(&mut self, world: &mut World, _events: &mut EventSystem) -> GameResult {
///         world.update();
///         Ok(())
///     }
///     
///     fn draw(&self, world: &World, ctx: &mut ggez::Context) -> GameResult {
///         world.draw(ctx)
///     }
///     
///     fn handle_input(&mut self, _world: &mut World, _events: &mut EventSystem, 
///                    _keycode: KeyCode, _ctx: &mut ggez::Context) {
///         // Handle input here
///     }
/// }
/// ```
pub trait SimulationState {
    fn update(&mut self, world: &mut World, events: &mut EventSystem) -> GameResult;
    fn draw(&self, world: &World, ctx: &mut Context) -> GameResult;
    fn handle_input(&mut self, world: &mut World, events: &mut EventSystem, keycode: KeyCode, ctx: &mut Context);
}

/// Example:
/// ```
/// use world_simulator::states::{SimulationState, RunningState};
/// use world_simulator::world::World;
/// use world_simulator::events::EventSystem;
/// use ggez::error::GameResult;
/// use ggez::event::KeyCode;
/// 
/// // Create the state and simulation objects
/// let mut state = RunningState::new();
/// let mut world = World::new(100, 100);
/// let mut events = EventSystem::new();
/// 
/// // Demonstrate state behavior (without actual rendering)
/// # fn test_state() -> GameResult {
/// #    let mut state = RunningState::new();
/// #    let mut world = World::new(100, 100);
/// #    let mut events = EventSystem::new();
/// #    state.update(&mut world, &mut events)?;
/// #    Ok(())
/// # }
/// # test_state().unwrap();
/// ```
#[derive(Default)]
pub struct RunningState {
    // Add any state-specific fields here
}

impl RunningState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SimulationState for RunningState {
    fn update(&mut self, world: &mut World, _events: &mut EventSystem) -> GameResult {
        world.update();
        Ok(())
    }

    fn draw(&self, world: &World, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        world.draw(&mut canvas, ctx)?;
        canvas.finish(ctx)?;
        Ok(())
    }

    fn handle_input(&mut self, _world: &mut World, events: &mut EventSystem, keycode: KeyCode, _ctx: &mut Context) {
        match keycode {
            KeyCode::Space => {
                events.emit(SimulationEvent::StateChanged(GameState::Paused));
            }
            _ => {}
        }
    }
}

#[derive(Default)]
pub struct PausedState {
    // Add any state-specific fields here
}

impl PausedState {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SimulationState for PausedState {
    fn update(&mut self, _world: &mut World, _events: &mut EventSystem) -> GameResult {
        // World is paused, no updates needed
        Ok(())
    }

    fn draw(&self, world: &World, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        world.draw(&mut canvas, ctx)?;
        canvas.finish(ctx)?;
        Ok(())
    }

    fn handle_input(&mut self, _world: &mut World, events: &mut EventSystem, keycode: KeyCode, _ctx: &mut Context) {
        match keycode {
            KeyCode::Space => {
                events.emit(SimulationEvent::StateChanged(GameState::Running));
            }
            _ => {}
        }
    }
}

/// The possible states of the simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    Running,
    Paused,
}

pub struct MenuState;

impl SimulationState for MenuState {
    fn update(&mut self, _world: &mut World, _event_system: &mut EventSystem) -> GameResult<()> {
        Ok(())
    }

    fn draw(&self, _world: &World, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn handle_input(&mut self, _world: &mut World, _event_system: &mut EventSystem, _keycode: KeyCode, _ctx: &mut Context) {
        // Handle input for menu state
    }
} 