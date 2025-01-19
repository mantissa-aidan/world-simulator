"""
Basic example demonstrating the use of the World Simulator Python bindings.
"""

from world_sim import PyWorld
import time

def main():
    # Create a world with 10 predators and 20 prey
    world = PyWorld(width=100, height=100, num_predators=10, num_prey=20)
    
    # Enable training mode for faster simulation
    world.enable_training_mode()
    
    # Run simulation for 1000 steps
    for i in range(1000):
        # Step the simulation and get agent positions
        positions = world.step()
        
        if i % 100 == 0:
            num_agents = len(positions)
            print(f"Step {i}: {num_agents} agents")
    
    # Reset the world
    world.reset()
    print("Simulation complete!")

if __name__ == "__main__":
    main() 