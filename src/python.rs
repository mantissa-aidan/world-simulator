use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use crate::world::World;
use crate::agent::AgentType;

/// Python wrapper for the World simulation
#[pyclass]
pub struct PyWorld {
    world: World,
    training_mode: bool,
}

#[pymethods]
impl PyWorld {
    #[new]
    fn new(width: i32, height: i32, num_predators: usize, num_prey: usize) -> Self {
        let mut world = World::new(width, height);
        world.initialize_agents(num_predators, num_prey);
        PyWorld {
            world,
            training_mode: false,
        }
    }

    /// Enable training mode for faster simulation
    fn enable_training_mode(&mut self) {
        self.world.enable_training_mode(None);
    }

    /// Disable training mode and return to normal simulation
    fn disable_training_mode(&mut self) {
        self.training_mode = false;
        self.world.disable_training_mode();
    }

    /// Step the simulation forward and return the new state
    fn step(&mut self) -> PyResult<()> {
        self.world.update();
        Ok(())
    }

    /// Reset the simulation to initial state
    fn reset(&mut self) -> PyResult<()> {
        self.world = World::new(self.world.width(), self.world.height());
        Ok(())
    }

    /// Get the current state of all agents
    fn get_agent_states(&self) -> PyResult<Vec<AgentState>> {
        let mut states = Vec::new();
        for agent in self.world.agents() {
            if let (Some(pos), Some(mov)) = (agent.position(), agent.movement()) {
                states.push(AgentState {
                    agent_type: match agent.agent_type {
                        AgentType::TypeA => 1, // Predator
                        AgentType::TypeB => 0, // Prey
                    },
                    x: pos.x,
                    y: pos.y,
                    velocity_x: mov.velocity.0,
                    velocity_y: mov.velocity.1,
                });
            }
        }
        Ok(states)
    }

    /// Get the current agent counts
    fn get_agent_counts(&self) -> PyResult<(usize, usize)> {
        Ok(self.world.get_agent_counts())
    }

    /// Get the dimensions of the world
    fn get_dimensions(&self) -> PyResult<(i32, i32)> {
        Ok((self.world.width(), self.world.height()))
    }

    /// Apply movement action to a specific agent
    fn apply_action(&mut self, agent_idx: usize, dx: f32, dy: f32) -> PyResult<bool> {
        Ok(self.world.apply_agent_action(agent_idx, dx, dy))
    }
}

/// Represents the state of a single agent
#[pyclass]
#[derive(Clone)]
pub struct AgentState {
    #[pyo3(get)]
    pub agent_type: i32,  // 1 for predator, 0 for prey
    #[pyo3(get)]
    pub x: f32,
    #[pyo3(get)]
    pub y: f32,
    #[pyo3(get)]
    pub velocity_x: f32,
    #[pyo3(get)]
    pub velocity_y: f32,
}

/// Python module for world_simulator
#[pymodule]
fn world_simulator(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyWorld>()?;
    m.add_class::<AgentState>()?;
    Ok(())
} 