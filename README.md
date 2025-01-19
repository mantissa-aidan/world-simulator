# World Simulator

A predator-prey simulation built in Rust using ggez for visualization. The simulation features agents that can chase, flee, and interact with each other in a 2D environment.

## Features

- Predator and prey agents with distinct behaviors
- Training mode for reinforcement learning
- Real-time visualization with ggez
- Configurable simulation parameters
- Performance optimizations including SIMD and batch processing

## Requirements

- Rust (stable channel)
- Cargo package manager
- ggez dependencies (see below for platform-specific requirements)

### Platform-specific Requirements

#### Windows
- OpenGL development libraries
- Visual Studio build tools

#### Linux
```bash
sudo apt-get install libasound2-dev libudev-dev pkg-config
```

#### macOS
```bash
brew install pkg-config
```

## Running the Simulation

### Normal Mode
Run the simulation with real-time visualization:
```bash
cargo run --release
```

### Training Mode
Run in training mode (faster updates, no rendering):
```bash
cargo run --release -- --training
```

### Debug Mode
Run with debug information and slower updates:
```bash
cargo run
```

## Testing

### Run All Tests
```bash
cargo test --release
```

### Run Documentation Tests
```bash
cargo test --doc --release
```

### Run Specific Tests
```bash
# Run performance benchmarks
cargo test world::tests::benchmark_training_mode -- --nocapture

# Run movement tests
cargo test world::tests::test_agent_movement -- --nocapture
```

## Controls

- `Space`: Toggle pause/resume simulation
- `Esc`: Exit simulation

## Configuration

The simulation can be configured through constants in `src/constants.rs`:

- `VISION_RANGE`: How far agents can see
- `AGENT_SPEED`: Base movement speed
- `WORLD_WIDTH/HEIGHT`: Simulation world dimensions
- `GAME_SPEED`: Update rate (FPS)

## Performance Tuning

For optimal performance when running in training mode:
```bash
RUSTFLAGS="-C target-cpu=native" cargo run --release -- --training
```

This enables CPU-specific optimizations including SIMD instructions where available.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
