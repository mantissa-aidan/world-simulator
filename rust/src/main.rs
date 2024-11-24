use ggez::graphics::{self, Color, DrawParam, Mesh};
use ggez::{Context, GameResult};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::graphics::{DrawParam as GgezDrawParam, Text};
use ggez::{ContextBuilder, GameError};
use ggez::conf::{WindowMode, WindowSetup};
use std::f32::consts::PI;
use std::time::{Duration, Instant};
use rayon::prelude::*;


const VISION_RANGE: f32 = 60.0; // Set your desired vision range here
const SCALING_FACTOR: f32 = 20.0; // Set your desired vision range here
const AGENT_DRAW_SIZE: f32 = 4.0;
const GAME_SPEED: u64 = 50; // Set your desired game speed here


// Represents an individual agent in the simulation.
#[derive(Clone, Copy)]
pub struct Agent {
    pub x: i32,
    pub y: i32,
    pub health: i32,
    pub agent_type: AgentType,
    pub direction: f32,
}

// Enum representing the type of agent.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AgentType {
    TypeA,
    TypeB,
}

impl Agent {
    // Creates a new agent with specified position and type.
    pub fn new(x: i32, y: i32, agent_type: AgentType) -> Self {
        Agent {
            x,
            y,
            health: 100,
            agent_type,
            direction: 0.0, // Initialize direction to 0 radians (facing right)
        }
    }

    // Rotates the agent by a given angle in radians.
    pub fn rotate(&mut self, angle: f32) {
        self.direction = (self.direction + angle) % (2.0 * PI);
    }

    // Moves the agent in the direction it is facing.
    pub fn move_forward(&mut self, max_x: i32, max_y: i32) {
        let dx = self.direction.cos().round() as i32;
        let dy = self.direction.sin().round() as i32;
        self.x = (self.x + dx) % max_x;
        self.y = (self.y + dy) % max_y;
    
        // Handle wrap-around for negative positions
        if self.x < 0 {
            self.x += max_x;
        }
        if self.y < 0 {
            self.y += max_y;
        }
    }
    
    pub fn can_see(&self, other: &Agent, fov: f32) -> bool {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dist = ((dx * dx + dy * dy) as f32).sqrt();
        if dist > VISION_RANGE {
            return false;
        }
        let angle_to_other = (dy as f32).atan2(dx as f32);
        let mut angle_diff = (angle_to_other - self.direction).abs();
        angle_diff = angle_diff % (2.0 * PI);
        if angle_diff > PI {
            angle_diff = 2.0 * PI - angle_diff;
        }
        angle_diff <= fov / 2.0
    }
    
    

    // Moves the agent randomly within the bounds of the world.
    pub fn move_randomly(&mut self, max_x: i32, max_y: i32) {
        use rand::{thread_rng, Rng};
        let mut rng = thread_rng();
        let angle = rng.gen_range(-0.5..=0.5);
        self.rotate(angle);
    
        // Move forward and apply wrap-around
        self.move_forward(max_x, max_y);
    }    
    

    pub fn move_towards(&mut self, target_x: f32, target_y: f32, max_x: i32, max_y: i32) {
        let target_dx = target_x - self.x as f32 * SCALING_FACTOR;
        let target_dy = target_y - self.y as f32 * SCALING_FACTOR;
        let angle_to_target = target_dy.atan2(target_dx);
        self.direction = angle_to_target;
    
        let speed = 1.0;
        let dx = (self.direction.cos() * speed).round() as i32;
        let dy = (self.direction.sin() * speed).round() as i32;
    
        self.x = (self.x + dx) % max_x;
        self.y = (self.y + dy) % max_y;
    
        if self.x < 0 {
            self.x += max_x;
        }
        if self.y < 0 {
            self.y += max_y;
        }
    }

    pub fn find_nearest_visible_agent<'a>(
        &self,
        agents: &'a [Agent],
        target_type: AgentType,
        grid: &SpatialGrid,
    ) -> Option<(usize, &'a Agent)> {
        let position = (self.x as f32 * SCALING_FACTOR, self.y as f32 * SCALING_FACTOR);
        let neighbor_indices = grid.get_neighbors(position);
    
        let mut nearest_agent: Option<(usize, &Agent)> = None;
        let mut min_dist = f32::MAX;
        for idx in neighbor_indices {
            let agent = &agents[idx];
            if agent.agent_type == target_type {
                if self.can_see(agent, PI / 4.0) {
                    let dx = agent.x - self.x;
                    let dy = agent.y - self.y;
                    let dist = ((dx * dx + dy * dy) as f32).sqrt();
                    if dist < min_dist {
                        min_dist = dist;
                        nearest_agent = Some((idx, agent));
                    }
                }
            }
        }
        nearest_agent
    }
    
    


    /// Moves away from a target position.
    pub fn move_away_from(&mut self, target_x: f32, target_y: f32, max_x: i32, max_y: i32) {
        let target_dx = self.x as f32 * SCALING_FACTOR - target_x;
        let target_dy = self.y as f32 * SCALING_FACTOR - target_y;
        let angle_away = target_dy.atan2(target_dx);
        self.direction = angle_away;
        self.move_forward(max_x, max_y);
    }
    
}

// Represents the world in which agents exist.
pub struct World {
    pub width: i32,
    pub height: i32,
    pub agents: Vec<Agent>,
}

impl World {
    // Creates a new world with specified width and height.
    pub fn new(width: i32, height: i32) -> Self {
        World {
            width,
            height,
            agents: Vec::new(),
        }
    }

    // Adds a new agent to the world at a specified position and type.
    pub fn add_agent(&mut self, x: i32, y: i32, agent_type: AgentType) {
        self.agents.push(Agent::new(x, y, agent_type));
    }

    pub fn update(&mut self, follow_mouse: bool, mouse_position: [f32; 2]) {
        // Build the spatial grid
        self.grid.clear();
        for (i, agent) in self.agents.iter().enumerate() {
            let position = (
                agent.x as f32 * SCALING_FACTOR,
                agent.y as f32 * SCALING_FACTOR,
            );
            self.grid.insert(position, i);
        }
    
        // Use a vector to collect indices of agents to remove
        let mut agents_to_remove = Vec::new();
    
        for (i, agent) in self.agents.iter_mut().enumerate() {
            if follow_mouse {
                agent.move_towards(
                    mouse_position[0],
                    mouse_position[1],
                    self.width,
                    self.height,
                );
            } else {
                match agent.agent_type {
                    AgentType::TypeA => {
                        // Predators look for prey
                        if let Some((prey_idx, prey)) =
                            agent.find_nearest_visible_agent(&self.agents, AgentType::TypeB, &self.grid)
                        {
                            // Move towards prey
                            agent.move_towards(
                                prey.x as f32 * SCALING_FACTOR,
                                prey.y as f32 * SCALING_FACTOR,
                                self.width,
                                self.height,
                            );
                            // Check if predator caught the prey
                            if agent.x == prey.x && agent.y == prey.y {
                                // Mark prey for removal
                                agents_to_remove.push(prey_idx);
                            }
                        } else {
                            // No prey in sight, move randomly
                            agent.move_randomly(self.width, self.height);
                        }
                    }
                    AgentType::TypeB => {
                        // Prey avoid predators
                        if let Some((_predator_idx, predator)) =
                            agent.find_nearest_visible_agent(&self.agents, AgentType::TypeA, &self.grid)
                        {
                            // Move away from predator
                            agent.move_away_from(
                                predator.x as f32 * SCALING_FACTOR,
                                predator.y as f32 * SCALING_FACTOR,
                                self.width,
                                self.height,
                            );
                        } else {
                            // No predator in sight, move randomly
                            agent.move_randomly(self.width, self.height);
                        }
                    }
                }
            }
        }
    
        // Remove captured prey
        agents_to_remove.sort_unstable();
        agents_to_remove.dedup();
        for index in agents_to_remove.into_iter().rev() {
            self.agents.remove(index);
        }
    }
    
    
    
    

    // Draws all agents in the world on the screen.
    pub fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        for agent in &self.agents {
            let color = match agent.agent_type {
                AgentType::TypeA => Color::BLUE,
                AgentType::TypeB => Color::RED,
            };
            let circle = Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [0.0, 0.0],
                AGENT_DRAW_SIZE,
                0.1,
                color,
            )?;
            graphics::draw(ctx, &circle, DrawParam::default().dest([agent.x as f32 * SCALING_FACTOR, agent.y as f32 * SCALING_FACTOR]))?;

            // Draw googly eyes
            let eye_offset = 4.0;
            let eye_radius = 2.5;

            // Calculate eye positions based on agent's direction
            let left_eye_x = agent.x as f32 * SCALING_FACTOR + eye_offset * agent.direction.cos() - eye_offset * agent.direction.sin();
            let left_eye_y = agent.y as f32 * SCALING_FACTOR + eye_offset * agent.direction.sin() + eye_offset * agent.direction.cos();

            let right_eye_x = agent.x as f32 * SCALING_FACTOR + eye_offset * agent.direction.cos() + eye_offset * agent.direction.sin();
            let right_eye_y = agent.y as f32 * SCALING_FACTOR + eye_offset * agent.direction.sin() - eye_offset * agent.direction.cos();

            let eye_color = Color::WHITE;

            let left_eye = Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [0.0, 0.0],
                eye_radius,
                0.1,
                eye_color,
            )?;
            let right_eye = Mesh::new_circle(
                ctx,
                graphics::DrawMode::fill(),
                [0.0, 0.0],
                eye_radius,
                0.1,
                eye_color,
            )?;

            graphics::draw(ctx, &left_eye, DrawParam::default().dest([left_eye_x, left_eye_y]))?;
            graphics::draw(ctx, &right_eye, DrawParam::default().dest([right_eye_x, right_eye_y]))?;

            // Draw the direction line (eye)
            let eye_length = SCALING_FACTOR;
            let eye_end_x = agent.x as f32 * SCALING_FACTOR + eye_length * agent.direction.cos();
            let eye_end_y = agent.y as f32 * SCALING_FACTOR + eye_length * agent.direction.sin();
            let eye_line = Mesh::new_line(
                ctx,
                &[[agent.x as f32 * SCALING_FACTOR, agent.y as f32 * SCALING_FACTOR], [eye_end_x, eye_end_y]],
                2.0,
                Color::WHITE,
            )?;
            graphics::draw(ctx, &eye_line, DrawParam::default())?;

            // Draw the vision cone
            let cone_length = VISION_RANGE; // Use VISION_RANGE for cone length
            let fov = PI / 4.0; // Field of view angle
            let left_angle = agent.direction - fov / 2.0;
            let right_angle = agent.direction + fov / 2.0;
            let left_end_x = agent.x as f32 * SCALING_FACTOR + cone_length * left_angle.cos();
            let left_end_y = agent.y as f32 * SCALING_FACTOR + cone_length * left_angle.sin();
            let right_end_x = agent.x as f32 * SCALING_FACTOR + cone_length * right_angle.cos();
            let right_end_y = agent.y as f32 * SCALING_FACTOR + cone_length * right_angle.sin();
        
            let cone = Mesh::new_polyline(
                ctx,
                graphics::DrawMode::stroke(1.0),
                &[
                    [agent.x as f32 * SCALING_FACTOR, agent.y as f32 * SCALING_FACTOR],
                    [left_end_x, left_end_y],
                    [right_end_x, right_end_y],
                    [agent.x as f32 * SCALING_FACTOR, agent.y as f32 * SCALING_FACTOR],
                ],
                Color::YELLOW,
            )?;
            graphics::draw(ctx, &cone, DrawParam::default())?;
        }
        Ok(())
    }
}

// Enum representing the possible states of the simulation.
enum GameState {
    Running,
    Paused,
}

// Main simulation structure.
struct Simulation {
    world: World,
    state: GameState,
    last_update: Instant,
    update_interval: Duration,
    follow_mouse: bool,
    mouse_position: [f32; 2],
}

impl Simulation {
    // Creates a new simulation with a predefined world and agents.

    fn new() -> Self {
        let mut world = World::new(160, 90); // Increased from 80x45 to 160x90
        // Add multiple predators and prey
        for _ in 0..100 {
            world.add_agent(rand::random::<i32>() % 160, rand::random::<i32>() % 90, AgentType::TypeA);
            world.add_agent(rand::random::<i32>() % 160, rand::random::<i32>() % 90, AgentType::TypeB);
        }

        Simulation { 
            world, 
            state: GameState::Running,
            last_update: Instant::now(),
            update_interval: Duration::from_millis(GAME_SPEED),
            follow_mouse: false,
            mouse_position: [0.0, 0.0],
        }
    }
    

    
    // Toggles the state of the simulation between Running and Paused.
    fn toggle_pause(&mut self) {
        self.state = match self.state {
            GameState::Running => GameState::Paused,
            GameState::Paused => GameState::Running,
        };
    }

    // Draws the pause menu on the screen.
    fn draw_pause_menu(&self, ctx: &mut Context) -> GameResult<()> {
        let resume_text = Text::new("Press 'R' to Resume");
        let quit_text = Text::new("Press 'Q' to Quit");
        let follow_mode_text = if self.follow_mouse {
            Text::new("Press 'F' to Disable Follow Mouse Mode")
        } else {
            Text::new("Press 'F' to Enable Follow Mouse Mode")
        };

        graphics::draw(ctx, &resume_text, GgezDrawParam::default().dest([100.0, 100.0]))?;
        graphics::draw(ctx, &quit_text, GgezDrawParam::default().dest([100.0, 150.0]))?;
        graphics::draw(ctx, &follow_mode_text, GgezDrawParam::default().dest([100.0, 200.0]))?;
        Ok(())
    }
}

// Implementation of the `EventHandler` trait for the `Simulation`.
impl EventHandler<GameError> for Simulation {
    // Update function, called on each frame.
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= self.update_interval {
            if let GameState::Running = self.state {
                self.world.update(self.follow_mouse, self.mouse_position);
            }
            self.last_update = now;
        }
        Ok(())
    }

    // Draw function, called on each frame to render the current state of the simulation.
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, Color::BLACK);

        match self.state {
            GameState::Running => self.world.draw(ctx)?,
            GameState::Paused => self.draw_pause_menu(ctx)?,
        }

        graphics::present(ctx)?;
        Ok(())
    }

    // Handles key down events to control the simulation state.
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match (&self.state, keycode) {
            (GameState::Running, KeyCode::P) => self.toggle_pause(),
            (GameState::Paused, KeyCode::R) => self.toggle_pause(),
            (GameState::Paused, KeyCode::Q) => event::quit(ctx),
            (GameState::Paused, KeyCode::F) => {
                self.follow_mouse = !self.follow_mouse;
            }
            _ => {},
        }
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.mouse_position = [x, y];
    }
}

fn main() -> GameResult {
    let (ctx, event_loop) = ContextBuilder::new("world_simulator", "YourName")
        .window_mode(WindowMode::default().dimensions(3200.0, 1800.0)) // Doubled window dimensions
        .window_setup(WindowSetup::default().title("World Simulator"))
        .build()?;

    let simulation = Simulation::new();
    event::run(ctx, event_loop, simulation)
}
