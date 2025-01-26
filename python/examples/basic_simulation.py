"""
Basic example demonstrating the use of the World Simulator Python bindings.
"""

from world_sim import PyWorld
import subprocess
import time
import sys

def main():
    # Start the simulation
    world = PyWorld(width=800, height=600, num_predators=10, num_prey=100)
    
    # Determine the path to the visualization binary
    cargo_cmd = "cargo.exe" if sys.platform == "win32" else "cargo"
    
    try:
        # Try to launch visualization in separate process
        print("Launching visualization...")
        vis_process = subprocess.Popen(
            [cargo_cmd, "run", "--release", "--bin", "world_simulator"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE
        )
        
        # Check if the process started successfully
        time.sleep(1)  # Give it a moment to start
        if vis_process.poll() is not None:
            # Process ended immediately, print the error
            out, err = vis_process.communicate()
            print("Error starting visualization:")
            print(err.decode())
            vis_process = None
        else:
            print("Visualization launched successfully!")
    except FileNotFoundError:
        print("Could not find cargo. Make sure Rust is installed and in your PATH")
        vis_process = None
    except Exception as e:
        print(f"Error launching visualization: {e}")
        vis_process = None
    
    try:
        # Run your simulation
        print("Starting simulation...")
        for i in range(1000):
            world.step()
            if i % 100 == 0:
                predators, prey = world.get_agent_counts()
                print(f"Step {i}: {predators} predators, {prey} prey")
    
    finally:
        # Clean up visualization process
        if vis_process:
            print("Closing visualization...")
            vis_process.terminate()
            vis_process.wait()

if __name__ == "__main__":
    main() 