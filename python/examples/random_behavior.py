"""
Simple example with random movement behavior for all agents.
"""

from world_sim import WorldEnv
import numpy as np

def main():
    # Create a small world for testing
    env = WorldEnv(width=100, height=100, num_predators=3, num_prey=5)
    
    # Get initial state
    states, predators, prey = env.reset()
    print(f"Starting with {predators} predators and {prey} prey")
    
    # Run simulation for 500 steps
    for step in range(500):
        # Generate random actions for each agent
        # Each action is a (dx, dy) tuple with values between -1 and 1
        num_agents = len(states)
        random_actions = []
        for _ in range(num_agents):
            # Generate random angle
            angle = np.random.uniform(0, 2 * np.pi)
            # Convert to direction vector
            dx = np.cos(angle)
            dy = np.sin(angle)
            random_actions.append((dx, dy))
        
        # Apply the random actions and get new state
        states, predators, prey = env.step(random_actions)
        
        # Print status every 100 steps
        if step % 100 == 0:
            print(f"Step {step}: {len(states)} agents ({predators} predators, {prey} prey)")
            # Print position of first predator and first prey if they exist
            for agent in states[:2]:  # Look at first two agents
                agent_type = "Predator" if agent.agent_type == 1 else "Prey"
                print(f"{agent_type} at position ({agent.x:.1f}, {agent.y:.1f})")

if __name__ == "__main__":
    main() 