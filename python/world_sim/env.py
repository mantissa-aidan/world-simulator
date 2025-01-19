import gymnasium as gym
import numpy as np
from gymnasium import spaces
from .world_simulator import PyWorld

class WorldSimEnv(gym.Env):
    """
    A Gymnasium environment for the World Simulator.
    
    This environment allows training agents in the predator-prey simulation
    using reinforcement learning algorithms.
    """
    metadata = {"render_modes": ["human", "rgb_array"], "render_fps": 30}

    def __init__(self, render_mode=None, config=None):
        self.config = {
            "width": 800,
            "height": 600,
            "num_predators": 10,
            "num_prey": 20,
            "max_steps": 1000,
        }
        if config:
            self.config.update(config)

        # Create the world simulator
        self.world = PyWorld(
            self.config["width"],
            self.config["height"],
            self.config["num_predators"],
            self.config["num_prey"]
        )
        
        # Enable training mode for better performance
        self.world.enable_training_mode()
        
        # Define action and observation spaces
        # Actions: [dx, dy] for each controlled agent
        self.action_space = spaces.Box(
            low=-1.0,
            high=1.0,
            shape=(2,),
            dtype=np.float32
        )
        
        # Observations: positions of all agents
        max_agents = self.config["num_predators"] + self.config["num_prey"]
        self.observation_space = spaces.Box(
            low=0,
            high=max(self.config["width"], self.config["height"]),
            shape=(max_agents * 2,),  # x,y for each agent
            dtype=np.float32
        )
        
        self.render_mode = render_mode
        self.steps = 0

    def reset(self, seed=None, options=None):
        """Reset the environment to initial state."""
        super().reset(seed=seed)
        
        # Reset the world
        self.world.reset()
        self.steps = 0
        
        # Get initial observation
        observation = self._get_observation()
        info = {}
        
        return observation, info

    def step(self, action):
        """
        Take a step in the environment.
        
        Args:
            action: numpy array [dx, dy] representing desired movement direction
            
        Returns:
            observation: Current state of the environment
            reward: Reward for the current step
            terminated: Whether the episode has ended
            truncated: Whether the episode was artificially terminated
            info: Additional information
        """
        # Apply action
        # TODO: Implement action application to agents
        
        # Step the world simulation
        positions = self.world.step()
        self.steps += 1
        
        # Get new observation
        observation = self._get_observation()
        
        # Calculate reward
        reward = self._calculate_reward()
        
        # Check if episode is done
        terminated = self._is_terminated()
        truncated = self.steps >= self.config["max_steps"]
        
        info = {}
        
        return observation, reward, terminated, truncated, info

    def render(self):
        """Render the environment."""
        # The world simulator already handles rendering
        pass

    def close(self):
        """Clean up resources."""
        pass

    def _get_observation(self):
        """Convert world state to observation array."""
        positions = self.world.get_agent_positions()
        return np.array(positions, dtype=np.float32).flatten()

    def _calculate_reward(self):
        """Calculate the reward for the current step."""
        # TODO: Implement proper reward calculation
        # For now, return a simple reward based on survival
        return 0.1

    def _is_terminated(self):
        """Check if the episode should terminate."""
        # TODO: Implement proper termination conditions
        return False 