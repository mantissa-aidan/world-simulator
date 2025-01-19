//! # Component System
//! 
//! This module implements the Component part of our Entity-Component System (ECS).
//! Components are pure data structures that can be attached to entities (Agents).
//!
//! ## Design Pattern: Component Pattern
//! The Component pattern allows us to:
//! - Add new behaviors without modifying existing code
//! - Compose complex entities from simple parts
//! - Share common functionality between different types of entities
//!
//! ## Implementation Details
//! - Components are trait objects that can be downcasted to concrete types
//! - Each component type represents a specific aspect of an entity
//! - Components are stored in a HashMap within each Agent
//!
//! ## Example
//! ```rust
//! use world_simulator::components::{Component, Position};
//!
//! let position = Position { x: 10, y: 20 };
//! // Can be added to any agent using agent.add_component(position)
//! ```

use std::any::Any;
use std::f32::consts::PI;

/// The base trait for all components.
/// 
/// This trait enables dynamic dispatch and type-safe downcasting of components.
/// It requires Send + Sync to enable parallel processing of agents.
pub trait Component: Send + Sync {
    /// Converts the component to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    /// Converts the component to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clone_box(&self) -> Box<dyn Component>;
}

/// Position component for storing agent location
#[derive(Clone)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Position { x, y }
    }
}

impl Component for Position {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

/// Movement component for storing agent velocity
#[derive(Clone)]
pub struct Movement {
    pub velocity: (f32, f32),
    pub speed: f32,  // Base movement speed
}

impl Movement {
    pub fn new(speed: f32) -> Self {
        Movement {
            velocity: (speed, 0.0),  // Initial velocity with defined speed
            speed,
        }
    }
}

impl Component for Movement {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
}

/// Health component for entities that can take damage
#[derive(Default, Clone)]
pub struct Health {
    pub value: i32,
}

impl Component for Health {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
    fn clone_box(&self) -> Box<dyn Component> { Box::new(self.clone()) }
}

/// Vision component for entities that can see others
#[derive(Clone)]
pub struct Vision {
    /// Maximum vision range
    pub range: f32,
    /// Field of view in radians
    pub fov: f32,
}

impl Vision {
    pub fn new(range: f32) -> Self {
        Vision {
            range,
            fov: PI / 4.0,  // Default 45-degree field of view
        }
    }
}

impl Component for Vision {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn Component> {
        Box::new(self.clone())
    }
} 