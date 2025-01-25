"""
Basic example demonstrating the use of the World Simulator Python bindings.
"""

from world_sim import PyWorld
import time

def main():
    # Create a world with larger populations
    scaling_factor = 10
    world = PyWorld(width=200 * scaling_factor, height=112 * scaling_factor, num_predators=50 * scaling_factor, num_prey=100 * scaling_factor)
    
    # Enable training mode for faster simulation
    world.enable_training_mode()
    
    # Run simulation for 1000 steps
    for i in range(10000):
        # Step the simulation
        world.step()
        
        if i % 100 == 0:
            # Get current state
            states = world.get_agent_states()
            #print(states)
            num_agents = len(states)
            predators, prey = world.get_agent_counts()
            print(f"Step {i}: {num_agents} agents ({predators} predators, {prey} prey)")
    
    # Reset the world
    world.reset()
    print("Simulation complete!")

if __name__ == "__main__":
    main() 