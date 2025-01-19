//! # State Management System
//! 
//! This module implements the State pattern to manage different simulation states
//! and their transitions. It provides a clean way to handle different behaviors
//! based on the current state of the simulation.
//!
//! ## Design Pattern: State
//!
//! The State pattern is implemented through:
//! - `SimulationState` trait defining state behavior
//! - `GameState` enum listing possible states
//! - Concrete state implementations for each game state
//!
//! ### Benefits
//! - Clean separation of state-specific behavior
//! - Easy to add new states without modifying existing code
//! - Clear state transition logic
//! - Eliminates complex conditional logic
//!
//! ### Example Usage
//! ```rust
//! use world_simulator::states::{SimulationState, GameState};
//!
//! // Create a state
//! struct RunningState;
//! impl SimulationState for RunningState {
//!     fn update(&mut self) {
//!         // Update simulation logic
//!     }
//!
//!     fn transition(&self) -> Option<GameState> {
//!         // Check conditions and transition if needed
//!         None
//!     }
//! }
//! ```

use ggez::{Context, GameResult};
use ggez::event::{self, KeyCode};
use ggez::graphics::{self, Text};

use crate::world::World;
use crate::events::{EventSystem, SimulationEvent};

/// Represents different states the simulation can be in
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GameState {
    /// Initial setup state
    Setup,
    /// Main simulation running state
    Running,
    /// Simulation is paused
    Paused,
    /// Simulation has ended
    GameOver,
    Menu,
}

/// Trait defining behavior for simulation states
pub trait SimulationState {
    /// Update logic for the current state
    fn update(&mut self, world: &mut World, event_system: &mut EventSystem) -> GameResult<()>;
    
    /// Check if state should transition to another state
    fn transition(&self) -> Option<GameState>;

    fn draw(&self, world: &World, ctx: &mut Context) -> GameResult<()>;
    fn handle_input(&mut self, world: &mut World, event_system: &mut EventSystem, keycode: KeyCode, ctx: &mut Context);
}

/// State when simulation is actively running
#[derive(Default)]
pub struct RunningState;

impl RunningState {
    /// Creates a new running state
    pub fn new(_world: World, _events: EventSystem) -> Self {
        RunningState
    }
}

impl SimulationState for RunningState {
    fn update(&mut self, _world: &mut World, _event_system: &mut EventSystem) -> GameResult<()> {
        // World update is handled by Simulation
        Ok(())
    }

    fn draw(&self, world: &World, ctx: &mut Context) -> GameResult<()> {
        world.draw(ctx)
    }

    fn handle_input(&mut self, _world: &mut World, event_system: &mut EventSystem, keycode: KeyCode, _ctx: &mut Context) {
        match keycode {
            KeyCode::P => event_system.emit(SimulationEvent::StateChanged(GameState::Paused)),
            _ => {},
        }
    }

    fn transition(&self) -> Option<GameState> {
        None
    }
}

/// State when simulation is paused
#[derive(Default)]
pub struct PausedState;

impl PausedState {
    /// Creates a new paused state
    pub fn new(_world: World, _events: EventSystem) -> Self {
        PausedState
    }
}

impl SimulationState for PausedState {
    fn update(&mut self, _world: &mut World, _event_system: &mut EventSystem) -> GameResult<()> {
        Ok(())
    }

    fn draw(&self, _world: &World, ctx: &mut Context) -> GameResult<()> {
        // Draw pause menu
        let resume_text = Text::new("Press 'R' to Resume");
        let quit_text = Text::new("Press 'Q' to Quit");
        let follow_text = Text::new("Press 'F' to Toggle Mouse Following");
        
        graphics::draw(ctx, &resume_text, graphics::DrawParam::default().dest([100.0, 100.0]))?;
        graphics::draw(ctx, &quit_text, graphics::DrawParam::default().dest([100.0, 150.0]))?;
        graphics::draw(ctx, &follow_text, graphics::DrawParam::default().dest([100.0, 200.0]))?;
        Ok(())
    }

    fn handle_input(&mut self, _world: &mut World, event_system: &mut EventSystem, keycode: KeyCode, ctx: &mut Context) {
        match keycode {
            KeyCode::R => event_system.emit(SimulationEvent::StateChanged(GameState::Running)),
            KeyCode::Q => event::quit(ctx),
            _ => {},
        }
    }

    fn transition(&self) -> Option<GameState> {
        None
    }
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

    fn transition(&self) -> Option<GameState> {
        None
    }
} 