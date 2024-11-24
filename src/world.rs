use ggez::graphics::{self, Color, DrawParam, Mesh};
use ggez::{Context, GameResult};
use rayon::prelude::*;


#[derive(Clone, Copy)]
pub struct Agent {
    pub x: i32,
    pub y: i32,
    pub health: i32,
    pub agent_type: AgentType,
}

#[derive(Clone, Copy)]
pub enum AgentType {
    TypeA,
    TypeB,
}

impl Agent {
    pub fn new(x: i32, y: i32, agent_type: AgentType) -> Self {
        Agent {
            x,
            y,
            health: 100,
            agent_type,
        }
    }

    // Simple random movement logic for the agent
    pub fn move_randomly(&mut self, max_x: i32, max_y: i32) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        self.x = (self.x + rng.gen_range(-1..=1)).clamp(0, max_x - 1);
        self.y = (self.y + rng.gen_range(-1..=1)).clamp(0, max_y - 1);
    }
}

pub struct World {
    pub width: i32,
    pub height: i32,
    pub agents: Vec<Agent>,
    grid: SpatialGrid

}

impl World {
    pub fn new(width: i32, height: i32) -> Self {
        let cell_size = VISION_RANGE;
        World {
            width,
            height,
            agents: Vec::new(),
            grid: SpatialGrid::new(cell_size),
        }
    }

    pub fn add_agent(&mut self, x: i32, y: i32, agent_type: AgentType) {
        self.agents.push(Agent::new(x, y, agent_type));
    }

    pub fn update(&mut self) {
        // Update each agent's position and interactions
        for agent in &mut self.agents {
            agent.move_randomly(self.width, self.height);
        }
    }

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
                10.0,
                0.1,
                color,
            )?;
            graphics::draw(ctx, &circle, DrawParam::default().dest([agent.x as f32 * 20.0, agent.y as f32 * 20.0]))?;
        }
        Ok(())
    }
}
