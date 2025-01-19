//! # Event System
//! 
//! This module implements the Observer pattern for event handling in the simulation.
//! It provides a decoupled way for components to communicate and react to changes.
//!
//! ## Design Pattern: Observer
//!
//! The Observer pattern is implemented with:
//! - `SimulationEvent` enum defining possible events
//! - `EventListener` trait for objects that want to receive events
//! - `EventSystem` managing subscriptions and event distribution
//!
//! ### Benefits
//! - Decoupled communication between components
//! - Easy to add new event types and listeners
//! - Centralized event handling
//!
//! ### Example Usage
//! ```rust
//! use world_simulator::events::{EventSystem, EventListener, SimulationEvent};
//!
//! #[derive(Clone)]
//! struct MyListener;
//!
//! impl EventListener for MyListener {
//!     fn on_event(&mut self, event: &SimulationEvent) {
//!         match event {
//!             SimulationEvent::AgentCollision(a, b) => println!("Collision between {} and {}", a, b),
//!             _ => {},
//!         }
//!     }
//!
//!     fn clone_box(&self) -> Box<dyn EventListener> {
//!         Box::new(self.clone())
//!     }
//! }
//!
//! // Set up the event system
//! let mut events = EventSystem::new();
//! events.add_listener("my_listener".to_string(), Box::new(MyListener));
//! ```

use std::collections::HashMap;
use crate::states::GameState;

/// Events that can occur during simulation
#[derive(Clone, Debug)]
pub enum SimulationEvent {
    /// When two agents collide (indices of colliding agents)
    AgentCollision(usize, usize),
    /// When an agent moves (index of moved agent)
    AgentMoved(usize),
    /// When simulation state changes
    StateChanged(GameState),
    /// When an agent dies
    AgentDied(usize),
    /// When a new agent is created
    AgentSpawned(usize),
}

/// Trait for objects that want to receive events
pub trait EventListener: Send + Sync {
    /// Called when an event occurs
    fn on_event(&mut self, event: &SimulationEvent);
    /// Clone implementation for trait object
    fn clone_box(&self) -> Box<dyn EventListener>;
}

impl Clone for Box<dyn EventListener> {
    fn clone(&self) -> Self {
        self.as_ref().clone_box()
    }
}

/// The event system that manages event distribution
#[derive(Default)]
pub struct EventSystem {
    /// Map of listener ID to listener implementation
    listeners: HashMap<String, Box<dyn EventListener>>,
}

impl Clone for EventSystem {
    fn clone(&self) -> Self {
        let mut new_listeners = HashMap::new();
        for (id, listener) in &self.listeners {
            new_listeners.insert(id.clone(), listener.clone());
        }
        EventSystem {
            listeners: new_listeners,
        }
    }
}

impl EventSystem {
    /// Creates a new event system
    pub fn new() -> Self {
        EventSystem {
            listeners: HashMap::new(),
        }
    }

    /// Adds a new event listener
    ///
    /// # Arguments
    /// * `id` - Unique identifier for the listener
    /// * `listener` - The listener implementation
    pub fn add_listener(&mut self, id: String, listener: Box<dyn EventListener>) {
        self.listeners.insert(id, listener);
    }

    /// Emits an event to all registered listeners
    ///
    /// # Arguments
    /// * `event` - The event to emit
    pub fn emit(&mut self, event: SimulationEvent) {
        for listener in self.listeners.values_mut() {
            listener.on_event(&event);
        }
    }
} 