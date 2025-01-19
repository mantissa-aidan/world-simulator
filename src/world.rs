//! # World System
//! 
//! This module implements the world simulation and manages agent interactions.
//! It coordinates agent updates and handles the main simulation loop.
//!
//! ## Design Patterns
//!
//! ### Parallel Processing
//! The world update system uses parallel processing:
//! - Agent decisions are computed in parallel using rayon
//! - Updates are applied in parallel to avoid race conditions
//! - Grid updates are synchronized to maintain consistency

use ggez::graphics::{self, Color, DrawParam, Mesh, DrawMode};
use ggez::{Context, GameResult};
use rayon::prelude::*;
use rand;
use std::time::Instant;

use crate::agent::{Agent, AgentType};
use crate::constants::*;
use crate::components::Vision;
use crate::spatial::SpatialGrid;
use crate::components::Position;
use crate::constants::CATCH_DISTANCE;

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// The main world struct that manages all agents and their interactions
#[derive(Clone)]
pub struct World {
    /// List of all agents in the world
    agents: Vec<Agent>,
    /// Spatial partitioning grid for efficient neighbor lookups
    grid: SpatialGrid,
    /// World dimensions
    width: i32,
    height: i32,
    /// Current simulation speed
    simulation_speed: u64,
    /// Training mode flag and settings
    training_mode: bool,
    training_config: TrainingConfig,
}

/// Configuration for training mode
#[derive(Clone)]
pub struct TrainingConfig {
    /// Skip visual updates to improve performance
    pub skip_rendering: bool,
    /// Number of simulation steps per action
    pub steps_per_action: u32,
    /// Maximum number of agents to consider for state observation
    pub max_observable_agents: usize,
    /// Maximum observation range for agents
    pub observation_range: f32,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        TrainingConfig {
            skip_rendering: true,
            steps_per_action: 1,
            max_observable_agents: 10,
            observation_range: VISION_RANGE,
        }
    }
}

impl World {
    pub fn new(width: i32, height: i32) -> Self {
        World {
            agents: Vec::new(),
            grid: SpatialGrid::new(
                (width as f32) * SCALING_FACTOR,
                (height as f32) * SCALING_FACTOR,
                VISION_RANGE,
            ),
            width,
            height,
            simulation_speed: DEFAULT_GAME_SPEED,
            training_mode: false,
            training_config: TrainingConfig::default(),
        }
    }

    /// Create a new world with specified number of predators and prey
    pub fn new_with_agents(width: i32, height: i32, num_predators: usize, num_prey: usize) -> Self {
        let mut world = Self::new(width, height);
        world.initialize_agents(num_predators, num_prey);
        world
    }

    /// Initialize or reset the world with specified number of agents
    pub fn initialize_agents(&mut self, num_predators: usize, num_prey: usize) {
        self.agents.clear();
        
        // Add predators
        for _ in 0..num_predators {
            self.add_agent(
                rand::random::<i32>() % self.width,
                rand::random::<i32>() % self.height,
                AgentType::TypeA,
            );
        }

        // Add prey
        for _ in 0..num_prey {
            self.add_agent(
                rand::random::<i32>() % self.width,
                rand::random::<i32>() % self.height,
                AgentType::TypeB,
            );
        }
    }

    /// Get current agent counts
    pub fn get_agent_counts(&self) -> (usize, usize) {
        let mut predators = 0;
        let mut prey = 0;
        
        for agent in &self.agents {
            match agent.agent_type {
                AgentType::TypeA => predators += 1,
                AgentType::TypeB => prey += 1,
            }
        }
        
        (predators, prey)
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn agents_mut(&mut self) -> &mut [Agent] {
        &mut self.agents
    }

    pub fn add_agent(&mut self, x: i32, y: i32, agent_type: AgentType) {
        let mut agent = Agent::new(x, y, agent_type);
        // Set speed based on agent type
        let speed = match agent_type {
            AgentType::TypeA => BASE_PREDATOR_SPEED,
            AgentType::TypeB => BASE_PREY_SPEED,
        };
        if let Some(mov) = agent.movement_mut() {
            mov.speed = speed;
        }
        self.agents.push(agent);
    }

    pub fn set_simulation_speed(&mut self, speed: f32) {
        // Convert 0.0-1.0 to game speed range
        let speed = (MIN_GAME_SPEED as f32 + (MAX_GAME_SPEED - MIN_GAME_SPEED) as f32 * (1.0 - speed)) as u64;
        self.simulation_speed = speed.clamp(MIN_GAME_SPEED, MAX_GAME_SPEED);
    }

    pub fn get_simulation_speed(&self) -> f32 {
        // Convert game speed to 0.0-1.0 range
        1.0 - (self.simulation_speed - MIN_GAME_SPEED) as f32 / (MAX_GAME_SPEED - MIN_GAME_SPEED) as f32
    }

    pub fn get_frame_time(&self) -> u64 {
        self.simulation_speed
    }

    /// Enable training mode with optional custom configuration
    pub fn enable_training_mode(&mut self, config: Option<TrainingConfig>) {
        self.training_mode = true;
        if let Some(cfg) = config {
            self.training_config = cfg;
        }
    }

    /// Disable training mode
    pub fn disable_training_mode(&mut self) {
        self.training_mode = false;
    }

    /// Get current training mode status
    pub fn is_training(&self) -> bool {
        self.training_mode
    }

    /// Update the training configuration
    pub fn set_training_config(&mut self, config: TrainingConfig) {
        self.training_config = config;
    }

    pub fn update(&mut self) {
        // In training mode, we might want to run multiple steps per update
        let steps = if self.training_mode {
            self.training_config.steps_per_action
        } else {
            1
        };

        for _ in 0..steps {
            self.update_step();
        }
    }

    /// Calculate distances between positions using SIMD (x86_64 with AVX support)
    #[cfg(target_arch = "x86_64")]
    unsafe fn calculate_distances_simd(&self, positions: &[(f32, f32)], target: (f32, f32)) -> Vec<f32> {
        use std::arch::x86_64::*;

        let len = positions.len();
        let mut distances = Vec::with_capacity(len);
        distances.set_len(len); // Pre-allocate to avoid reallocation

        // Process 8 positions at a time using AVX
        let target_x = _mm256_set1_ps(target.0);
        let target_y = _mm256_set1_ps(target.1);

        let chunks = len / 8;
        let remainder = len % 8;

        for i in 0..chunks {
            let base_idx = i * 8;
            let mut x_coords = [0.0f32; 8];
            let mut y_coords = [0.0f32; 8];

            // Gather coordinates into arrays
            for j in 0..8 {
                x_coords[j] = positions[base_idx + j].0;
                y_coords[j] = positions[base_idx + j].1;
            }

            // Load coordinates into SIMD registers
            let pos_x = _mm256_loadu_ps(x_coords.as_ptr());
            let pos_y = _mm256_loadu_ps(y_coords.as_ptr());

            // Calculate differences
            let dx = _mm256_sub_ps(pos_x, target_x);
            let dy = _mm256_sub_ps(pos_y, target_y);

            // Calculate squared distances
            let dx_squared = _mm256_mul_ps(dx, dx);
            let dy_squared = _mm256_mul_ps(dy, dy);
            let sum = _mm256_add_ps(dx_squared, dy_squared);

            // Calculate square root
            let dist = _mm256_sqrt_ps(sum);

            // Store results
            _mm256_storeu_ps(&mut distances[base_idx], dist);
        }

        // Handle remaining elements
        for i in (chunks * 8)..len {
            let dx = positions[i].0 - target.0;
            let dy = positions[i].1 - target.1;
            distances[i] = (dx * dx + dy * dy).sqrt();
        }

        distances
    }

    /// Process catches using SIMD for distance calculations
    #[cfg(target_arch = "x86_64")]
    fn process_catches_simd(&mut self) -> Vec<usize> {
        const BATCH_SIZE: usize = 64;
        let mut caught_indices = Vec::new();

        // Collect predator and prey positions
        let mut predator_positions = Vec::new();
        let mut prey_positions = Vec::new();
        let mut prey_indices = Vec::new();

        for (idx, agent) in self.agents.iter().enumerate() {
            if let Some(pos) = agent.position() {
                match agent.agent_type {
                    AgentType::TypeA => predator_positions.push((pos.x, pos.y)),
                    AgentType::TypeB => {
                        prey_positions.push((pos.x, pos.y));
                        prey_indices.push(idx);
                    }
                }
            }
        }

        // Process predators in batches
        for chunk in predator_positions.chunks(BATCH_SIZE) {
            for &pred_pos in chunk {
                // Safety: calculate_distances_simd is unsafe due to SIMD instructions
                let distances = unsafe { self.calculate_distances_simd(&prey_positions, pred_pos) };
                
                // Check for catches
                for (i, &dist) in distances.iter().enumerate() {
                    if dist <= CATCH_RADIUS {
                        caught_indices.push(prey_indices[i]);
                    }
                }
            }
        }

        caught_indices.sort_unstable();
        caught_indices.dedup();
        caught_indices
    }

    // Fallback implementation for non-x86_64 platforms
    #[cfg(not(target_arch = "x86_64"))]
    fn process_catches_simd(&mut self) -> Vec<usize> {
        let mut caught_indices = Vec::new();
        
        for (i, agent_a) in self.agents.iter().enumerate() {
            if agent_a.agent_type == AgentType::TypeA {
                if let Some(pos_a) = agent_a.position() {
                    for (j, agent_b) in self.agents.iter().enumerate() {
                        if agent_b.agent_type == AgentType::TypeB {
                            if let Some(pos_b) = agent_b.position() {
                                let dx = pos_a.x - pos_b.x;
                                let dy = pos_a.y - pos_b.y;
                                let distance = (dx * dx + dy * dy).sqrt();
                                if distance <= CATCH_RADIUS {
                                    caught_indices.push(j);
                                }
                            }
                        }
                    }
                }
            }
        }

        caught_indices.sort_unstable();
        caught_indices.dedup();
        caught_indices
    }

    /// Single step of simulation update
    fn update_step(&mut self) {
        const BATCH_SIZE: usize = 64;  // Process agents in batches for better cache utilization

        // Update spatial grid with current agent positions
        self.grid.clear();
        self.agents.iter().enumerate().for_each(|(idx, agent)| {
            if let Some(pos) = agent.position() {
                self.grid.insert((pos.x, pos.y), idx);
            }
        });

        // Calculate speed adjustment based on simulation speed
        let speed_factor = DEFAULT_GAME_SPEED as f32 / self.simulation_speed as f32;

        // Process agents in batches
        let agent_moves: Vec<_> = (0..self.agents.len())
            .collect::<Vec<_>>()
            .chunks(BATCH_SIZE)
            .flat_map(|batch| {
                batch.par_iter().map(|&agent_idx| {
                    let agent = &self.agents[agent_idx];
                    let agent_pos = if let Some(pos) = agent.position() {
                        pos
                    } else {
                        return (agent_idx, None);
                    };

                    // Get nearby agents for collision avoidance - reuse allocated vector
                    let nearby_indices = self.grid.get_nearby((agent_pos.x, agent_pos.y), MIN_AGENT_DISTANCE * 2.0);
                    
                    // Use stack allocation for small arrays
                    let mut avoid_vec = [(0.0f32, 0.0f32); 8];
                    let mut avoid_count = 0;

                    // Calculate avoidance vector with smooth falloff
                    for &other_idx in &nearby_indices {
                        if other_idx == agent_idx || avoid_count >= avoid_vec.len() {
                            continue;
                        }
                        if let Some(other_pos) = self.agents[other_idx].position() {
                            let dx = agent_pos.x - other_pos.x;
                            let dy = agent_pos.y - other_pos.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist < MIN_AGENT_DISTANCE * 2.0 {
                                let strength = 1.0 - (dist / (MIN_AGENT_DISTANCE * 2.0)).min(1.0);
                                avoid_vec[avoid_count] = (dx / dist * strength, dy / dist * strength);
                                avoid_count += 1;
                            }
                        }
                    }

                    // Find nearest agent of opposite type for chase/flee behavior
                    let target_pos = if let Some((_, other)) = agent.find_nearest_visible_agent(
                        &self.agents,
                        match agent.agent_type {
                            AgentType::TypeA => AgentType::TypeB,
                            AgentType::TypeB => AgentType::TypeA,
                        },
                        &self.grid,
                    ) {
                        if let Some(other_pos) = other.position() {
                            match agent.agent_type {
                                AgentType::TypeA => Some((other_pos.x, other_pos.y)),
                                AgentType::TypeB => {
                                    let dx = agent_pos.x - other_pos.x;
                                    let dy = agent_pos.y - other_pos.y;
                                    Some((agent_pos.x + dx, agent_pos.y + dy))
                                }
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Combine target movement with collision avoidance
                    if avoid_count > 0 {
                        let avoid_scale = 1.0;
                        if let Some((target_x, target_y)) = target_pos {
                            let dx = target_x - agent_pos.x;
                            let dy = target_y - agent_pos.y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist > 0.0 {
                                let target_weight = 1.0 - (avoid_count as f32 * SMOOTHING_FACTOR).min(1.0);
                                
                                // Sum up avoidance vectors
                                let mut avoid_x = 0.0;
                                let mut avoid_y = 0.0;
                                for i in 0..avoid_count {
                                    avoid_x += avoid_vec[i].0;
                                    avoid_y += avoid_vec[i].1;
                                }
                                
                                let mix_x = dx / dist * target_weight + avoid_scale * avoid_x / avoid_count as f32;
                                let mix_y = dy / dist * target_weight + avoid_scale * avoid_y / avoid_count as f32;
                                let mix_len = (mix_x * mix_x + mix_y * mix_y).sqrt();
                                if mix_len > 0.0 {
                                    let final_x = agent_pos.x + mix_x / mix_len;
                                    let final_y = agent_pos.y + mix_y / mix_len;
                                    (agent_idx, Some((final_x, final_y)))
                                } else {
                                    (agent_idx, None)
                                }
                            } else {
                                (agent_idx, None)
                            }
                        } else {
                            // Pure avoidance
                            let mut avoid_x = 0.0;
                            let mut avoid_y = 0.0;
                            for i in 0..avoid_count {
                                avoid_x += avoid_vec[i].0;
                                avoid_y += avoid_vec[i].1;
                            }
                            let avoid_len = (avoid_x * avoid_x + avoid_y * avoid_y).sqrt();
                            if avoid_len > 0.0 {
                                (agent_idx, Some((
                                    agent_pos.x + avoid_x / avoid_len,
                                    agent_pos.y + avoid_y / avoid_len
                                )))
                            } else {
                                (agent_idx, None)
                            }
                        }
                    } else {
                        (agent_idx, target_pos)
                    }
                }).collect::<Vec<_>>()
            })
            .collect();

        // Apply moves in parallel with speed adjustment using chunks for better cache utilization
        self.agents.par_chunks_mut(BATCH_SIZE).for_each(|batch| {
            for agent in batch {
                let base_speed = match agent.agent_type {
                    AgentType::TypeA => BASE_PREDATOR_SPEED,
                    AgentType::TypeB => BASE_PREY_SPEED,
                } * speed_factor;

                if let Some(mov) = agent.movement_mut() {
                    mov.speed = base_speed;
                }
            }
        });

        // Apply movement in batches
        for (idx, target) in agent_moves {
            if let Some((target_x, target_y)) = target {
                self.agents[idx].move_towards(target_x, target_y, self.width as f32, self.height as f32);
            } else {
                self.agents[idx].move_randomly(self.width as f32, self.height as f32);
            }
        }

        // Replace the old catch processing with SIMD version
        let caught_indices = self.process_catches_simd();
        
        // Remove caught agents (in reverse order to maintain indices)
        for &idx in caught_indices.iter().rev() {
            self.agents.remove(idx);
        }
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        // Skip rendering in training mode if configured
        if self.training_mode && self.training_config.skip_rendering {
            return Ok(());
        }

        for agent in &self.agents {
            self.draw_agent(agent, ctx)?;
        }
        Ok(())
    }

    fn draw_agent(&self, agent: &Agent, ctx: &mut Context) -> GameResult<()> {
        if let (Some(pos), Some(mov), Some(_vision)) = (
            agent.position(),
            agent.movement(),
            agent.get_component::<Vision>()
        ) {
            let pos_x = pos.x * SCALING_FACTOR;
            let pos_y = pos.y * SCALING_FACTOR;
            let eye_offset = AGENT_DRAW_SIZE / 4.0;

            // Calculate agent's facing direction from velocity
            let direction = if mov.velocity.0 != 0.0 || mov.velocity.1 != 0.0 {
                mov.velocity.1.atan2(mov.velocity.0)
            } else {
                0.0 // Default direction if not moving
            };

            // Draw agent body
            let body = Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                [pos_x, pos_y],
                AGENT_DRAW_SIZE,
                0.1,
                match agent.agent_type {
                    AgentType::TypeA => Color::RED,
                    AgentType::TypeB => Color::BLUE,
                },
            )?;
            graphics::draw(ctx, &body, DrawParam::default())?;

            // Draw eyes
            let cos = direction.cos();
            let sin = direction.sin();
            
            // Left eye
            let left_eye = Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                [
                    pos_x + eye_offset * cos - eye_offset * sin,
                    pos_y + eye_offset * sin + eye_offset * cos,
                ],
                AGENT_DRAW_SIZE / 4.0,
                0.1,
                Color::WHITE,
            )?;
            graphics::draw(ctx, &left_eye, DrawParam::default())?;

            // Right eye
            let right_eye = Mesh::new_circle(
                ctx,
                DrawMode::fill(),
                [
                    pos_x + eye_offset * cos + eye_offset * sin,
                    pos_y + eye_offset * sin - eye_offset * cos,
                ],
                AGENT_DRAW_SIZE / 4.0,
                0.1,
                Color::WHITE,
            )?;
            graphics::draw(ctx, &right_eye, DrawParam::default())?;
        }
        Ok(())
    }

    fn process_catches(&mut self) {
        let mut prey_positions = Vec::new();
        let mut predator_positions = Vec::new();
        let mut predator_indices = Vec::new();
        let mut prey_indices = Vec::new();

        // Collect positions
        for (i, agent) in self.agents.iter().enumerate() {
            if let Some(pos) = agent.get_component::<Position>() {
                let pos_tuple = (pos.x, pos.y);
                match agent.agent_type {
                    AgentType::TypeA => {
                        predator_positions.push(pos_tuple);
                        predator_indices.push(i);
                    }
                    AgentType::TypeB => {
                        prey_positions.push(pos_tuple);
                        prey_indices.push(i);
                    }
                }
            }
        }

        let mut catches = Vec::new();

        // Check for catches using SIMD
        for (pred_idx, &pred_pos) in predator_indices.iter().zip(predator_positions.iter()) {
            let distances = unsafe { self.calculate_distances_simd(&prey_positions, pred_pos) };
            
            for (i, &distance) in distances.iter().enumerate() {
                if distance < CATCH_DISTANCE {
                    catches.push((*pred_idx, prey_indices[i]));
                }
            }
        }

        // Process catches
        for (pred_idx, prey_idx) in catches {
            // TODO: Implement catch logic
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_movement() {
        let mut world = World::new(100, 100);
        
        // Add a test agent
        world.add_agent(50, 50, AgentType::TypeA);
        
        // Get initial position
        let initial_pos = world.agents[0].position().unwrap().clone();
        
        // Update world several times
        for _ in 0..10 {
            world.update();
        }
        
        // Get final position
        let final_pos = world.agents[0].position().unwrap().clone();
        
        // Verify that the position has changed
        assert!(
            initial_pos.x != final_pos.x || initial_pos.y != final_pos.y,
            "Agent position did not change after updates. Initial: ({}, {}), Final: ({}, {})",
            initial_pos.x, initial_pos.y, final_pos.x, final_pos.y
        );
    }

    #[test]
    fn test_training_mode_configuration() {
        let mut world = World::new(100, 100);
        assert!(!world.is_training(), "Training mode should be disabled by default");

        // Test default configuration
        world.enable_training_mode(None);
        assert!(world.is_training(), "Training mode should be enabled");
        assert!(world.training_config.skip_rendering, "Default config should skip rendering");
        assert_eq!(world.training_config.steps_per_action, 1, "Default steps per action should be 1");
        assert_eq!(world.training_config.max_observable_agents, 10, "Default max observable agents should be 10");
        assert_eq!(world.training_config.observation_range, VISION_RANGE, "Default observation range should match VISION_RANGE");

        // Test custom configuration
        let custom_config = TrainingConfig {
            skip_rendering: false,
            steps_per_action: 4,
            max_observable_agents: 5,
            observation_range: 30.0,
        };
        world.set_training_config(custom_config.clone());
        assert_eq!(world.training_config.skip_rendering, false, "Custom rendering setting not applied");
        assert_eq!(world.training_config.steps_per_action, 4, "Custom steps per action not applied");
        assert_eq!(world.training_config.max_observable_agents, 5, "Custom max observable agents not applied");
        assert_eq!(world.training_config.observation_range, 30.0, "Custom observation range not applied");

        // Test disabling training mode
        world.disable_training_mode();
        assert!(!world.is_training(), "Training mode should be disabled");
    }

    #[test]
    fn test_training_mode_updates() {
        let mut world = World::new(100, 100);
        world.add_agent(50, 50, AgentType::TypeA);
        
        // Enable training mode with multiple steps per action
        let config = TrainingConfig {
            skip_rendering: true,
            steps_per_action: 3,
            max_observable_agents: 10,
            observation_range: VISION_RANGE,
        };
        world.enable_training_mode(Some(config));

        // Get initial position
        let initial_pos = world.agents[0].position().unwrap().clone();
        
        // Single update should perform multiple steps
        world.update();
        
        // Get final position
        let final_pos = world.agents[0].position().unwrap().clone();
        
        // Position should change more significantly due to multiple steps
        let distance = ((final_pos.x - initial_pos.x).powi(2) + 
                       (final_pos.y - initial_pos.y).powi(2)).sqrt();
        assert!(distance > 0.0, "Agent should move during training updates");
    }

    #[test]
    fn test_observation_limits() {
        let mut world = World::new(100, 100);
        
        // Add multiple agents close together
        for i in 0..5 {
            world.add_agent(50 + i, 50, AgentType::TypeA);
            world.add_agent(50 + i, 51, AgentType::TypeB);
        }

        // Enable training mode with limited observations
        let config = TrainingConfig {
            skip_rendering: true,
            steps_per_action: 1,
            max_observable_agents: 3,
            observation_range: 10.0,
        };
        world.enable_training_mode(Some(config));

        // Update world to process agent observations
        world.update();

        // Verify that agents can only see within the configured range
        if let Some(agent) = world.agents.first() {
            if let Some(_) = agent.get_component::<Vision>() {
                // Find nearest agent of opposite type
                let nearest = agent.find_nearest_visible_agent(
                    &world.agents,
                    match agent.agent_type {
                        AgentType::TypeA => AgentType::TypeB,
                        AgentType::TypeB => AgentType::TypeA,
                    },
                    &world.grid
                );
                
                // Verify that we can find a nearby agent (should be within observation range)
                assert!(nearest.is_some(), "Should find at least one visible agent");
                
                // Get the distance to the nearest agent
                if let Some((dist, _)) = nearest {
                    assert!(dist <= world.training_config.observation_range, 
                        "Nearest agent should be within observation range");
                }
            }
        }
    }

    #[test]
    fn benchmark_training_mode() {
        let mut world = World::new_with_agents(100, 100, 50, 50);
        let num_iterations = 1000;

        // Benchmark 1: Normal mode
        let start = Instant::now();
        for _ in 0..num_iterations {
            world.update();
        }
        let normal_time = start.elapsed();
        println!("Normal mode - {} iterations: {:?}", num_iterations, normal_time);

        // Benchmark 2: Training mode with default config
        world.enable_training_mode(None);
        let start = Instant::now();
        for _ in 0..num_iterations {
            world.update();
        }
        let training_default_time = start.elapsed();
        println!("Training mode (default) - {} iterations: {:?}", num_iterations, training_default_time);

        // Benchmark 3: Training mode with high steps per action
        let steps_per_action = 4;
        let config = TrainingConfig {
            skip_rendering: true,
            steps_per_action,
            max_observable_agents: 5,
            observation_range: 30.0,
        };
        world.set_training_config(config);
        let start = Instant::now();
        // Adjust iterations to match total steps
        let fast_iterations = num_iterations / steps_per_action;
        for _ in 0..fast_iterations {
            world.update();
        }
        let training_fast_time = start.elapsed();
        println!("Training mode (fast) - {} iterations ({} total steps): {:?}", 
            fast_iterations, fast_iterations * steps_per_action, training_fast_time);

        // Verify that training mode is faster
        assert!(training_default_time <= normal_time, 
            "Training mode should be faster than normal mode");
        
        // Print performance comparison (accounting for steps per action)
        println!("\nPerformance comparison (per simulation step):");
        println!("Normal mode: 1.0x (baseline)");
        println!("Training mode (default): {:.2}x speed",
            normal_time.as_secs_f64() / training_default_time.as_secs_f64());
        println!("Training mode (fast): {:.2}x speed",
            (normal_time.as_secs_f64() * steps_per_action as f64) / 
            (training_fast_time.as_secs_f64() * steps_per_action as f64));
    }

    #[test]
    fn test_agent_initialization() {
        let world = World::new_with_agents(100, 100, 30, 70);
        let (predators, prey) = world.get_agent_counts();
        assert_eq!(predators, 30, "Should have correct number of predators");
        assert_eq!(prey, 70, "Should have correct number of prey");

        // Test resetting with new counts
        let mut world = World::new(100, 100);
        world.initialize_agents(10, 20);
        let (predators, prey) = world.get_agent_counts();
        assert_eq!(predators, 10, "Should have updated number of predators");
        assert_eq!(prey, 20, "Should have updated number of prey");
    }

    #[test]
    fn benchmark_distance_calculations() {
        let world = World::new(100, 100);
        let mut positions = Vec::with_capacity(10000);
        
        // Generate test data
        for i in 0..10000 {
            let x = (i % 100) as f32;
            let y = (i / 100) as f32;
            positions.push((x, y));
        }
        
        let target = (50.0, 50.0);

        // Warm up the cache
        for _ in 0..5 {
            let _ = unsafe { world.calculate_distances_simd(&positions, target) };
            let mut scalar_distances = Vec::with_capacity(positions.len());
            for pos in &positions {
                let dx = pos.0 - target.0;
                let dy = pos.1 - target.1;
                scalar_distances.push((dx * dx + dy * dy).sqrt());
            }
        }

        // Benchmark SIMD implementation
        let start = Instant::now();
        let simd_distances = unsafe { world.calculate_distances_simd(&positions, target) };
        let simd_time = start.elapsed();

        // Benchmark scalar implementation
        let start = Instant::now();
        let mut scalar_distances = Vec::with_capacity(positions.len());
        for pos in &positions {
            let dx = pos.0 - target.0;
            let dy = pos.1 - target.1;
            scalar_distances.push((dx * dx + dy * dy).sqrt());
        }
        let scalar_time = start.elapsed();

        println!("Data size: {} points", positions.len());
        println!("SIMD time: {:?}", simd_time);
        println!("Scalar time: {:?}", scalar_time);
        println!("Speedup: {:.2}x", scalar_time.as_secs_f64() / simd_time.as_secs_f64());

        // Verify results
        for (simd_dist, scalar_dist) in simd_distances.iter().zip(scalar_distances.iter()) {
            assert!((simd_dist - scalar_dist).abs() < 1e-5);
        }
    }
}
