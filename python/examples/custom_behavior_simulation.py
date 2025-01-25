"""
Example demonstrating custom agent behaviors controlled from Python.
"""

from world_sim import WorldEnv
import numpy as np
import math

def calculate_agent_actions(states):
    """
    Calculate actions for all agents based on their current states.
    This is where you implement your custom behavior logic.
    """
    actions = []
    
    for agent in states:
        if agent.agent_type == 1:  # Predator
            # Example predator behavior: move in circles
            angle = math.atan2(agent.velocity_y, agent.velocity_x)
            angle += 0.1  # Rotate slowly
            actions.append((math.cos(angle), math.sin(angle)))
            
        else:  # Prey
            # Example prey behavior: move randomly
            angle = np.random.uniform(0, 2 * np.pi)
            actions.append((math.cos(angle), math.sin(angle)))
    
    return actions

def main():
    # Create world with smaller numbers for testing
    env = WorldEnv(width=100, height=100, num_predators=5, num_prey=10)
    
    # Run simulation with custom behaviors
    states, predators, prey = env.reset()
    
    for i in range(1000):
        # Calculate custom actions for each agent
        actions = calculate_agent_actions(states)
        
        # Step simulation with our actions
        states, predators, prey = env.step(actions)
        
        if i % 100 == 0:
            print(f"Step {i}: {len(states)} agents ({predators} predators, {prey} prey)")
    
    print("Simulation complete!")

if __name__ == "__main__":
    main() 