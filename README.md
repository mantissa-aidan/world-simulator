# World Simulator

A Rust-based simulation demonstrating modern game architecture and design patterns. This project serves as a learning resource for Rust programming concepts and software design patterns.

## Features

- Entity-Component System (ECS) architecture
- Spatial partitioning for efficient agent interactions
- Event-driven communication using the Observer pattern
- State management using the State pattern
- Parallel processing for performance optimization

## Architecture

The project is organized into several modules:

- `agent.rs`: Implements the Entity-Component System for agents
- `components.rs`: Defines component traits and implementations
- `events.rs`: Handles event management using the Observer pattern
- `states.rs`: Manages simulation states using the State pattern
- `world.rs`: Implements spatial partitioning and world management
- `main.rs`: Entry point and simulation setup

### Design Patterns

1. **Entity-Component System (ECS)**
   - Separates data from behavior
   - Enables flexible agent composition
   - Makes adding new features easier

2. **Observer Pattern (Events)**
   - Decouples communication between components
   - Centralizes event handling
   - Makes the system more maintainable

3. **State Pattern**
   - Cleanly manages different simulation states
   - Simplifies state transitions
   - Eliminates complex conditional logic

4. **Spatial Partitioning**
   - Optimizes spatial queries
   - Reduces collision detection complexity
   - Enables efficient neighbor lookups

## Getting Started

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/yourusername/world-simulator.git
cd world-simulator
```

2. Build the project:
```bash
cargo build
```

3. Run the simulation:
```bash
cargo run
```

### Controls

- Space: Pause/Resume simulation
- Esc: Exit to menu
- Arrow keys: Navigate menu
- Enter: Select menu option

## Learning Resources

This project demonstrates several key Rust concepts:

1. **Ownership and Borrowing**
   - Smart pointer usage (Box, Rc, Arc)
   - Lifetime management
   - Reference rules

2. **Trait System**
   - Trait objects for components
   - Trait bounds and generics
   - Dynamic dispatch

3. **Concurrency**
   - Parallel processing with Rayon
   - Thread-safe data structures
   - Message passing

4. **Module System**
   - Code organization
   - Visibility rules
   - Package management

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.
