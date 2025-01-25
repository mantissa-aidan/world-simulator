//! # Agent System
//! 
//! This module implements the Entity part of our Entity-Component System (ECS).
//! Agents are entities that can hold various components and interact with each other.
//!
//! ## Design Patterns
//!
//! ### Entity-Component System
//! - Agents are entities that store and manage components
//! - Components contain the actual data and behavior
//! - This separation allows for flexible entity composition
//!
//! ### Type-Safe Component Management
//! - Components are stored in a type-map (HashMap<TypeId, Box<dyn Component>>)
//! - Safe downcasting is provided through the Any trait
//! - Component access is checked at runtime
//!
//! ## Example Usage
//! ```rust
//! use world_simulator::agent::{Agent, AgentType};
//!
//! // Create a new agent
//! let mut agent = Agent::new(10, 20, AgentType::TypeA);
//!
//! // Access components
//! if let Some(position) = agent.position() {
//!     println!("Agent is at ({}, {})", position.x, position.y);
//! }
//! ```

use std::collections::HashMap;
use std::any::TypeId;
use rand::Rng;

use crate::constants::{VISION_RANGE, BASE_PREDATOR_SPEED, BASE_PREY_SPEED, MAX_ROTATION};
use crate::components::{Component, Position, Movement, Vision};
use crate::spatial::SpatialGrid;

/// Represents different types of agents in the simulation
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    /// Type A agents (typically predators)
    TypeA,
    /// Type B agents (typically prey)
    TypeB,
}

/// The main agent struct representing an entity in the simulation
pub struct Agent {
    /// Component storage using TypeId as key
    components: HashMap<TypeId, Box<dyn Component>>,
    /// The type of this agent
    pub agent_type: AgentType,
}

impl Clone for Agent {
    fn clone(&self) -> Self {
        let mut new_components = HashMap::new();
        for (type_id, component) in &self.components {
            new_components.insert(*type_id, component.clone_box());
        }
        Agent {
            components: new_components,
            agent_type: self.agent_type,
        }
    }
}

impl Agent {
    /// Creates a new agent with basic components
    ///
    /// # Arguments
    /// * `agent_type` - The type of agent to create
    pub fn new(agent_type: AgentType) -> Self {
        Agent {
            agent_type,
            components: HashMap::new(),
        }
    }

    /// Adds a component to the agent
    ///
    /// # Type Parameters
    /// * `T` - The component type that implements Component
    pub fn add_component<T: Component + 'static>(&mut self, component: T) {
        self.components.insert(TypeId::of::<T>(), Box::new(component));
    }

    /// Gets a reference to a component
    ///
    /// # Type Parameters
    /// * `T` - The component type to retrieve
    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        self.components.get(&TypeId::of::<T>())
            .and_then(|c| c.as_any().downcast_ref::<T>())
    }

    /// Gets a mutable reference to a component
    ///
    /// # Type Parameters
    /// * `T` - The component type to retrieve
    pub fn get_component_mut<T: Component + 'static>(&mut self) -> Option<&mut T> {
        self.components.get_mut(&TypeId::of::<T>())
            .and_then(|c| c.as_any_mut().downcast_mut::<T>())
    }

    pub fn position(&self) -> Option<&Position> {
        self.get_component::<Position>()
    }

    pub fn position_mut(&mut self) -> Option<&mut Position> {
        self.get_component_mut::<Position>()
    }

    pub fn movement(&self) -> Option<&Movement> {
        self.get_component::<Movement>()
    }

    pub fn movement_mut(&mut self) -> Option<&mut Movement> {
        self.get_component_mut::<Movement>()
    }

    pub fn rotate(&mut self, angle: f32) {
        if let Some(mov) = self.movement_mut() {
            let (vx, vy) = mov.velocity;
            let cos = angle.cos();
            let sin = angle.sin();
            mov.velocity = (
                vx * cos - vy * sin,
                vx * sin + vy * cos
            );
        }
    }

    pub fn move_forward(&mut self, max_x: f32, max_y: f32) {
        // Get movement first to avoid multiple borrows
        let velocity = if let Some(mov) = self.movement() {
            mov.velocity
        } else {
            return;
        };

        // Then update position
        if let Some(pos) = self.position_mut() {
            pos.x = (pos.x + velocity.0).rem_euclid(max_x);
            pos.y = (pos.y + velocity.1).rem_euclid(max_y);
        }
    }

    pub fn move_randomly(&mut self, world_width: f32, world_height: f32) {
        if let Some(mov) = self.movement_mut() {
            let mut rng = rand::thread_rng();
            let rotation = rng.gen_range(-MAX_ROTATION..=MAX_ROTATION);
            let (vx, vy) = mov.velocity;
            let speed = mov.speed;
            
            // Rotate current velocity
            let angle = vy.atan2(vx) + rotation;
            mov.velocity = (speed * angle.cos(), speed * angle.sin());
        }
        self.update_position(world_width, world_height);
    }

    pub fn move_towards(&mut self, target_x: f32, target_y: f32, world_width: f32, world_height: f32) {
        // Get current position and movement first to avoid multiple borrows
        let (current_x, current_y, current_speed) = if let (Some(pos), Some(mov)) = (self.position(), self.movement()) {
            (pos.x, pos.y, mov.speed)
        } else {
            return;
        };

        // Calculate direction
        let dx = target_x - current_x;
        let dy = target_y - current_y;
        let dist = (dx * dx + dy * dy).sqrt();
        
        // Calculate new velocity
        let new_velocity = if dist > 0.0 {
            (current_speed * dx / dist, current_speed * dy / dist)
        } else {
            (0.0, 0.0)
        };

        // Update movement
        if let Some(mov) = self.movement_mut() {
            mov.velocity = new_velocity;
        }

        // Update position
        self.update_position(world_width, world_height);
    }

    pub fn can_see(&self, other: &Agent, fov: f32) -> bool {
        if let (Some(self_pos), Some(other_pos), Some(self_mov)) = (
            self.position(),
            other.position(),
            self.movement()
        ) {
            let pos1 = SpatialGrid::to_world_space((self_pos.x, self_pos.y));
            let pos2 = SpatialGrid::to_world_space((other_pos.x, other_pos.y));
            
            if !SpatialGrid::is_within_range(pos1, pos2, VISION_RANGE) {
                return false;
            }

            let current_angle = self_mov.velocity.1.atan2(self_mov.velocity.0);
            SpatialGrid::is_within_fov(pos1, pos2, current_angle, fov)
        } else {
            false
        }
    }

    pub fn find_nearest_visible_agent<'a>(
        &self,
        agents: &'a [Agent],
        target_type: AgentType,
        grid: &SpatialGrid,
    ) -> Option<(f32, &'a Agent)> {
        let pos = self.position()?;
        let vision = self.get_component::<Vision>()?;

        let nearby_indices = grid.get_nearby((pos.x, pos.y), vision.range);
        
        nearby_indices
            .iter()
            .filter_map(|&idx| {
                let other = &agents[idx];
                if other.agent_type != target_type {
                    return None;
                }
                
                let other_pos = other.position()?;
                let dx = other_pos.x - pos.x;
                let dy = other_pos.y - pos.y;
                let dist = (dx * dx + dy * dy).sqrt();
                
                if dist <= vision.range {
                    Some((dist, other))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
    }

    /// Updates the agent's position based on its movement
    pub fn update_position(&mut self, world_width: f32, world_height: f32) {
        // Get velocity first to avoid multiple borrows
        let velocity = if let Some(mov) = self.movement() {
            mov.velocity
        } else {
            return;
        };

        // Then update position
        if let Some(pos) = self.position_mut() {
            pos.x = (pos.x + velocity.0).rem_euclid(world_width);
            pos.y = (pos.y + velocity.1).rem_euclid(world_height);
        }
    }

    /// Updates the agent's movement based on its behavior
    pub fn update_movement(&mut self) {
        // Get components first to avoid multiple mutable borrows
        let position = if let Some(pos) = self.get_component_mut::<Position>() {
            Some(pos.clone())
        } else {
            None
        };
        
        if let (Some(_pos), Some(mov)) = (
            position,
            self.get_component_mut::<Movement>()
        ) {
            // Update movement based on position and other factors
            // This is just an example - you'll want to implement your own movement logic
            mov.velocity = (0.0, 0.0);
        }
    }
} 