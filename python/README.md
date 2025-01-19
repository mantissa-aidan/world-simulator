# World Simulator Python Package

Python bindings for the Rust-based predator-prey simulation environment, designed for reinforcement learning applications.

## Installation

```bash
pip install -e .
```

## Usage

```python
from world_sim import PyWorld

# Create a world with 10 predators and 20 prey
world = PyWorld(width=100, height=100, num_predators=10, num_prey=20)

# Enable training mode for faster simulation
world.enable_training_mode()

# Run simulation
for i in range(1000):
    positions = world.step()  # Get agent positions
    
# Reset the world
world.reset()
```

## Features

- Fast Rust-based simulation engine
- Training mode for reinforcement learning
- Configurable world size and agent counts
- Simple Python interface

## Requirements

- Python >= 3.7
- numpy >= 1.20.0
- gymnasium >= 0.26.0 (for future RL integration) 