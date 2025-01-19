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

use crate::agent::{Agent, AgentType};
use crate::constants::*;
use crate::components::Vision;
use crate::spatial::SpatialGrid;

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
    simulation_speed: u64,  // Current simulation speed
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
        }
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

    pub fn update(&mut self) {
        // Update spatial grid with current agent positions
        self.grid.clear();
        for (idx, agent) in self.agents.iter().enumerate() {
            if let Some(pos) = agent.position() {
                self.grid.insert((pos.x, pos.y), idx);
            }
        }

        // Calculate speed adjustment based on simulation speed
        let speed_factor = DEFAULT_GAME_SPEED as f32 / self.simulation_speed as f32;

        // Collect all agent decisions in parallel
        let agent_moves: Vec<_> = (0..self.agents.len())
            .into_par_iter()
            .map(|agent_idx| {
                let agent = &self.agents[agent_idx];
                let agent_pos = if let Some(pos) = agent.position() {
                    pos
                } else {
                    return (agent_idx, None);
                };

                // Get nearby agents for collision avoidance
                let nearby_indices = self.grid.get_nearby((agent_pos.x, agent_pos.y), MIN_AGENT_DISTANCE * 2.0);
                let mut avoid_x = 0.0;
                let mut avoid_y = 0.0;
                let mut avoid_count = 0;

                // Calculate avoidance vector with smooth falloff
                for &other_idx in &nearby_indices {
                    if other_idx == agent_idx {
                        continue;
                    }
                    if let Some(other_pos) = self.agents[other_idx].position() {
                        let dx = agent_pos.x - other_pos.x;
                        let dy = agent_pos.y - other_pos.y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist < MIN_AGENT_DISTANCE * 2.0 {
                            let strength = 1.0 - (dist / (MIN_AGENT_DISTANCE * 2.0)).min(1.0);
                            avoid_x += dx / dist * strength;
                            avoid_y += dy / dist * strength;
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
                            AgentType::TypeA => Some((other_pos.x, other_pos.y)), // Chase
                            AgentType::TypeB => {
                                // Flee (move in opposite direction)
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
                    let avoid_scale = 1.0;  // Adjust this to control avoidance strength
                    if let Some((target_x, target_y)) = target_pos {
                        // Mix avoidance with target direction using smooth interpolation
                        let dx = target_x - agent_pos.x;
                        let dy = target_y - agent_pos.y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist > 0.0 {
                            let target_weight = 1.0 - (avoid_count as f32 * SMOOTHING_FACTOR).min(1.0);
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
                        // Pure avoidance with smooth interpolation
                        let avoid_x = avoid_x / avoid_count as f32;
                        let avoid_y = avoid_y / avoid_count as f32;
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
                    // No nearby agents, just use target position
                    (agent_idx, target_pos)
                }
            })
            .collect();

        // Apply moves in parallel with speed adjustment
        self.agents.par_iter_mut().enumerate().for_each(|(idx, agent)| {
            let base_speed = match agent.agent_type {
                AgentType::TypeA => BASE_PREDATOR_SPEED,
                AgentType::TypeB => BASE_PREY_SPEED,
            } * speed_factor;

            if let Some(mov) = agent.movement_mut() {
                mov.speed = base_speed;
            }

            if let Some((_, Some((target_x, target_y)))) = agent_moves.get(idx) {
                agent.move_towards(*target_x, *target_y, self.width as f32, self.height as f32);
            } else {
                agent.move_randomly(self.width as f32, self.height as f32);
            }
        });

        // Check for catches (Type A catching Type B)
        let mut caught_indices = Vec::new();
        for (i, agent_a) in self.agents.iter().enumerate() {
            if agent_a.agent_type != AgentType::TypeA {
                continue;
            }
            if let Some(pos_a) = agent_a.position() {
                for (j, agent_b) in self.agents.iter().enumerate() {
                    if agent_b.agent_type != AgentType::TypeB {
                        continue;
                    }
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

        // Remove caught agents (in reverse order to maintain indices)
        caught_indices.sort_unstable();
        caught_indices.dedup();
        for &idx in caught_indices.iter().rev() {
            self.agents.remove(idx);
        }
    }

    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
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
}
