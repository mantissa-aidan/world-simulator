//! # World Simulator
//! 
//! A Rust-based simulation that demonstrates various design patterns and Rust concepts.
//! This project implements a simple ecosystem where agents interact with each other in a 2D world.
//!
//! ## Design Patterns
//!
//! ### Component Pattern
//! The simulation uses an Entity-Component System (ECS) pattern where agents are entities that
//! hold various components (Position, Movement, Health, etc.). This allows for:
//! - Flexible agent composition
//! - Easy addition of new behaviors
//! - Better separation of concerns
//!
//! ### State Pattern
//! Game states (Running, Paused, Menu) are implemented using the State pattern, allowing:
//! - Clean separation of state-specific logic
//! - Easy addition of new states
//! - Centralized state management
//!
//! ### Observer Pattern
//! The event system implements the Observer pattern, enabling:
//! - Decoupled communication between components
//! - Easy addition of new event types
//! - Centralized event handling
//!
//! ## Key Rust Concepts Demonstrated
//!
//! - **Traits**: Used extensively for components and state management
//! - **Generics**: Used in component storage and retrieval
//! - **Lifetimes**: Used in agent references and component borrowing
//! - **Smart Pointers**: Used for component storage (Box)
//! - **Parallel Processing**: Using rayon for parallel agent updates
//! - **Module System**: Organized code structure with clear separation
//! - **Type System**: Leveraging Rust's type system for safety
//!
//! ## Module Structure
//!
//! - `agent`: Agent implementation with component management
//! - `components`: Component traits and implementations
//! - `events`: Event system implementation
//! - `states`: Game state management
//! - `world`: World and spatial grid implementation
//! - `constants`: Configuration constants
//! - `spatial`: Spatial module

pub mod agent;
pub mod components;
pub mod constants;
pub mod events;
pub mod spatial;
pub mod states;
pub mod world;
pub mod python;

use std::time::Instant;
use ggez::{Context, GameResult};
use ggez::event::{self, EventHandler};
use ggez::input::keyboard::{KeyInput, KeyCode};
use ggez::input::mouse::MouseButton;
use ggez::graphics::{self, Color, DrawMode, DrawParam, Mesh, Text, Canvas};
use std::time::Duration;

// Re-export commonly used items
pub use agent::{Agent, AgentType};
pub use states::{GameState, SimulationState};
pub use events::{EventSystem, SimulationEvent};

// Re-export the Python module
pub use crate::python::*;

/// Main simulation struct that manages the game state
pub struct Simulation {
    /// The world containing all agents
    world: world::World,
    /// Last update timestamp
    last_update: Instant,
    slider_rect: Option<Mesh>,
    slider_value: f32,  // 0.0 to 1.0
    slider_active: bool,
}

impl Simulation {
    pub fn new(width: i32, height: i32) -> Self {
        let mut world = world::World::new(width, height);

        // Create initial agents
        for _ in 0..100 {
            world.add_agent(
                agent::AgentType::TypeA,
                rand::random::<f32>() * width as f32,
                rand::random::<f32>() * height as f32,
            );
            world.add_agent(
                agent::AgentType::TypeB,
                rand::random::<f32>() * width as f32,
                rand::random::<f32>() * height as f32,
            );
        }

        Simulation {
            world,
            last_update: Instant::now(),
            slider_rect: None,
            slider_value: 0.5,  // Start at 50% speed
            slider_active: false,
        }
    }

    /// Enable training mode with optional custom configuration
    pub fn enable_training_mode(&mut self, config: Option<world::TrainingConfig>) {
        self.world.enable_training_mode(config);
    }

    /// Disable training mode
    pub fn disable_training_mode(&mut self) {
        self.world.disable_training_mode();
    }

    /// Get current training mode status
    pub fn is_training(&self) -> bool {
        self.world.is_training()
    }

    fn update_slider(&mut self, x: f32, y: f32) {
        if y >= 20.0 && y <= 40.0 && x >= 20.0 && x <= 220.0 {
            self.slider_value = ((x - 20.0) / 200.0).clamp(0.0, 1.0);
            self.world.set_simulation_speed(self.slider_value);
        }
    }
}

impl EventHandler<ggez::GameError> for Simulation {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let now = std::time::Instant::now();
        if now - self.last_update >= Duration::from_millis(self.world.get_frame_time()) {
            self.world.update();
            self.last_update = now;
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Skip drawing if in training mode with rendering disabled
        if self.world.is_training() {
            return Ok(());
        }

        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        self.world.draw(&mut canvas, ctx)?;

        // Draw slider background
        if let Some(ref slider_rect) = self.slider_rect {
            canvas.draw(slider_rect, DrawParam::default());
        }

        // Draw slider handle
        let handle = Mesh::new_circle(
            ctx,
            DrawMode::fill(),
            [20.0 + self.slider_value * 200.0, 30.0],
            8.0,
            0.1,
            Color::WHITE,
        )?;
        canvas.draw(&handle, DrawParam::default());

        // Draw speed text
        let speed_text = Text::new(format!("Speed: {:.0}%", self.slider_value * 100.0));
        canvas.draw(
            &speed_text,
            DrawParam::default().dest([230.0, 20.0]).color(Color::WHITE),
        );

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeat: bool) -> GameResult<()> {
        if input.keycode == Some(KeyCode::Q) {
            ctx.request_quit();
        }
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        x: f32,
        y: f32,
    ) -> GameResult<()> {
        if button == MouseButton::Left {
            // Check if click is in slider area
            if y >= 20.0 && y <= 40.0 && x >= 20.0 && x <= 220.0 {
                self.slider_active = true;
                self.slider_value = ((x - 20.0) / 200.0).clamp(0.0, 1.0);
            }
        }
        Ok(())
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) -> GameResult<()> {
        if button == MouseButton::Left {
            self.slider_active = false;
        }
        Ok(())
    }

    fn mouse_motion_event(
        &mut self,
        _ctx: &mut Context,
        x: f32,
        y: f32,
        _dx: f32,
        _dy: f32,
    ) -> GameResult<()> {
        if self.slider_active {
            self.slider_value = ((x - 20.0) / 200.0).clamp(0.0, 1.0);
        }
        Ok(())
    }
} 