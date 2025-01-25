import gymnasium as gym
import numpy as np
from gymnasium import spaces
from .world_simulator import PyWorld

class WorldSimEnv(gym.Env):
    """
    A Gymnasium environment for the World Simulator that uses direct simulation data.
    
    Instead of using rendered frames, this environment provides direct access to:
    - Agent positions and velocities
    - Agent types (predator/prey)
    - Distances to nearest agents
    - Current agent states
    """
    
    def __init__(self, config=None):
        self.config = {
            "width": 800,
            "height": 600,
            "num_predators": 10,
            "num_prey": 20,
            "max_steps": 1000,
            "max_observable_agents": 5,  # Number of closest agents to observe
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
        
        # Define action space: [dx, dy] for controlled agent
        self.action_space = spaces.Box(
            low=-1.0,
            high=1.0,
            shape=(2,),
            dtype=np.float32
        )
        
        # Define observation space
        # For each observable agent we track:
        # - Relative position (x, y)
        # - Relative velocity (dx, dy)
        # - Agent type (predator=1, prey=0)
        # - Distance
        n_features = 6  # [rel_x, rel_y, rel_dx, rel_dy, type, distance]
        n_observable = self.config["max_observable_agents"]
        
        self.observation_space = spaces.Dict({
            # Position and velocity of the controlled agent
            "self": spaces.Box(
                low=np.array([0, 0, -1, -1]),
                high=np.array([self.config["width"], self.config["height"], 1, 1]),
                shape=(4,),
                dtype=np.float32
            ),
            # Information about observable agents
            "others": spaces.Box(
                low=-float('inf'),
                high=float('inf'),
                shape=(n_observable, n_features),
                dtype=np.float32
            ),
            # Additional state information
            "stats": spaces.Dict({
                "num_predators": spaces.Discrete(self.config["num_predators"] + 1),
                "num_prey": spaces.Discrete(self.config["num_prey"] + 1),
            })
        })
        
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
            observation: Dictionary containing direct simulation data
            reward: Reward for the current step
            terminated: Whether the episode has ended
            truncated: Whether the episode was artificially terminated
            info: Additional information
        """
        # Get current state to find controlled agent
        agent_states = self.world.get_agent_states()
        controlled_idx = next(
            (i for i, agent in enumerate(agent_states) if agent.agent_type == 1),  # 1 = predator
            None
        )
        
        if controlled_idx is not None:
            # Apply action to controlled agent
            self.world.apply_action(controlled_idx, float(action[0]), float(action[1]))
        
        # Step the world simulation
        self.world.step()
        self.steps += 1
        
        # Get new observation
        observation = self._get_observation()
        
        # Calculate reward based on changes in agent counts
        reward = self._calculate_reward()
        
        # Check if episode is done
        terminated = self._is_terminated()
        truncated = self.steps >= self.config["max_steps"]
        
        # Gather additional info
        info = {
            "steps": self.steps,
            "predators_remaining": observation["stats"]["num_predators"],
            "prey_remaining": observation["stats"]["num_prey"]
        }
        
        return observation, reward, terminated, truncated, info

    def _get_observation(self):
        """Get the current state of the simulation as structured observation data."""
        # Get all agent states
        agent_states = self.world.get_agent_states()
        
        # Get controlled agent state (first predator for now)
        controlled_agent = next(
            (agent for agent in agent_states if agent.agent_type == 1),  # 1 = predator
            None
        )
        
        if controlled_agent is None:
            # No predators left, return zero observation
            return self._get_zero_observation()
            
        # Get self state
        self_state = np.array([
            controlled_agent.x,
            controlled_agent.y,
            controlled_agent.velocity_x,
            controlled_agent.velocity_y
        ], dtype=np.float32)
        
        # Get other agents relative to controlled agent
        other_states = []
        for agent in agent_states:
            if agent != controlled_agent:
                # Calculate relative position
                rel_x = agent.x - controlled_agent.x
                rel_y = agent.y - controlled_agent.y
                
                # Calculate relative velocity
                rel_dx = agent.velocity_x - controlled_agent.velocity_x
                rel_dy = agent.velocity_y - controlled_agent.velocity_y
                
                # Calculate distance
                distance = np.sqrt(rel_x * rel_x + rel_y * rel_y)
                
                other_states.append([
                    rel_x,
                    rel_y,
                    rel_dx,
                    rel_dy,
                    float(agent.agent_type),  # 1.0 for predator, 0.0 for prey
                    distance
                ])
        
        # Sort by distance and take closest n_observable
        other_states.sort(key=lambda x: x[5])  # Sort by distance
        other_states = other_states[:self.config["max_observable_agents"]]
        
        # Pad with zeros if needed
        while len(other_states) < self.config["max_observable_agents"]:
            other_states.append([0.0] * 6)
        
        # Get current counts
        num_predators, num_prey = self.world.get_agent_counts()
        
        return {
            "self": self_state,
            "others": np.array(other_states, dtype=np.float32),
            "stats": {
                "num_predators": num_predators,
                "num_prey": num_prey
            }
        }
        
    def _get_zero_observation(self):
        """Return a zero-filled observation."""
        n_observable = self.config["max_observable_agents"]
        n_features = 6
        
        return {
            "self": np.zeros(4, dtype=np.float32),
            "others": np.zeros((n_observable, n_features), dtype=np.float32),
            "stats": {
                "num_predators": 0,
                "num_prey": 0
            }
        }

    def _calculate_reward(self):
        """Calculate the reward based on simulation state."""
        # TODO: Implement proper reward calculation based on:
        # - Successful catches for predators
        # - Survival time for prey
        # - Distance to targets
        # - Energy efficiency
        return 0.0

    def _is_terminated(self):
        """Check if the episode should terminate."""
        # TODO: Implement proper termination conditions:
        # - All prey caught
        # - All predators starved
        # - Other win/lose conditions
        return False 