# World Simulator

A Rust-based simulation that demonstrates predator-prey dynamics in a 2D world, with Python bindings for machine learning integration.

## Features

- Real-time 2D simulation with predator and prey agents
- Entity-Component System (ECS) architecture
- SIMD-optimized distance calculations for performance
- Spatial partitioning for efficient neighbor lookups
- Python bindings for machine learning integration
- Training mode for faster simulation during learning
- Configurable simulation parameters
- Smooth agent movement with collision avoidance
- Visual feedback with agent direction indicators

## Requirements

- Rust (latest stable version)
- Python 3.6 or higher
- NumPy
- Gymnasium (for RL environment integration)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/world-simulator.git
cd world-simulator
```

2. Install the Python package in development mode:
```bash
cd python
pip install -e .
```

## Usage

### Python Interface

```python
from world_simulator import PyWorld

# Create a new world with dimensions 800x600 and some agents
world = PyWorld(800, 600, num_predators=10, num_prey=20)

# Enable training mode for faster simulation
world.enable_training_mode()

# Step the simulation and get agent positions
positions = world.step()

# Reset the simulation
world.reset()
```

### Running the Simulation

The simulation can be run directly using Cargo:

```bash
cargo run
```

Controls:
- Space: Pause/Resume simulation
- Q: Quit
- Mouse: Adjust simulation speed using the slider

## Architecture

The project uses several design patterns and Rust features:

### Component Pattern (ECS)
- Agents are entities that hold various components
- Components include Position, Movement, Vision
- Flexible agent composition and behavior

### State Pattern
- Different simulation states (Running, Paused, Menu)
- Clean separation of state-specific logic
- Centralized state management

### Observer Pattern
- Event system for decoupled communication
- Handles state changes and agent interactions

### Performance Optimizations
- SIMD-accelerated distance calculations
- Spatial partitioning for efficient neighbor lookups
- Batch processing for better cache utilization
- Parallel agent updates using rayon

## Project Structure

- `src/`
  - `agent.rs`: Agent implementation with component management
  - `components.rs`: Component traits and implementations
  - `events.rs`: Event system implementation
  - `states.rs`: Game state management
  - `world.rs`: World and spatial grid implementation
  - `constants.rs`: Configuration constants
  - `spatial.rs`: Spatial partitioning module
  - `python.rs`: Python bindings
- `python/`: Python package files

## TODO

### Core Features
- [ ] Add energy/health system for agents
- [ ] Implement agent reproduction mechanics
- [ ] Add food/resource spawning system
- [ ] Create different agent species with unique traits

### Technical Improvements
- [ ] Add OpenGL/WebGL rendering backend option
- [ ] Implement multi-threaded spatial partitioning
- [ ] Add configurable neural networks for agent behavior
- [ ] Create serialization system for saving/loading simulation states

### Machine Learning Integration
- [ ] Implement Gymnasium environment interface
- [ ] Add reward shaping options for RL training
- [ ] Create example training scripts for different RL algorithms
- [ ] Add tools for visualizing learned behaviors

### Documentation
- [ ] Add API documentation for Python bindings
- [ ] Create examples for common use cases
- [ ] Add performance tuning guide
- [ ] Create contribution guidelines

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)

## License

This project is licensed under the MIT License - see the LICENSE file for details.
