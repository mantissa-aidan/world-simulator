from world_sim import PyWorld
import subprocess
import time
import numpy as np

def main():
    # Create a world with interesting agent counts
    world = PyWorld(width=200, height=112, num_predators=20, num_prey=100)
    
    # Launch visualization in separate process
    vis_process = subprocess.Popen([
        "cargo", "run", "--release", "--bin", "world_simulator_vis",
        "200", "112"  # width and height must match world dimensions
    ])
    
    try:
        # Run simulation with some control
        for i in range(2000):
            # Get current state
            states = world.get_agent_states()
            predators, prey = world.get_agent_counts()
            
            # Optional: Add some controlled behavior
            if i % 100 == 0:
                # Every 100 steps, make predators move towards center
                for idx, state in enumerate(states):
                    if state.agent_type == 1:  # Predator
                        # Calculate direction to center
                        dx = 100 - state.x  # Center x
                        dy = 56 - state.y   # Center y
                        # Normalize
                        mag = np.sqrt(dx*dx + dy*dy)
                        if mag > 0:
                            dx, dy = dx/mag, dy/mag
                            world.apply_action(idx, dx, dy)
            
            # Step simulation
            world.step()
            
            # Print status every 100 steps
            if i % 100 == 0:
                print(f"Step {i}: {predators} predators, {prey} prey")
            
            # Optional: Add small delay to better see the visualization
            time.sleep(0.01)
    
    finally:
        # Clean up visualization process
        if vis_process:
            vis_process.terminate()
            vis_process.wait()

if __name__ == "__main__":
    main()