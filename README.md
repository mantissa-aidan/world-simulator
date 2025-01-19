# World Simulator

A simple world simulation written in Rust using the `ggez` game engine. This simulator includes agents of different types interacting in a 2D grid. Agents can move randomly, follow the mouse, chase prey, or avoid predators.

## Features
- **Agent Interaction**: Simulates predator-prey dynamics where predators chase prey and prey avoid predators.
- **Mouse Interaction**: Optionally allows agents to follow the mouse position.
- **Vision Cone**: Agents have a field of view that determines which other agents they can see.
- **Dynamic Updates**: Real-time simulation with pause and resume functionality.
- **Googly Eyes**: Agents are visualized with googly eyes and a vision cone for added fun!

---

## Installation Instructions

### 1. **Install Rust**  
Make sure you have Rust and Cargo installed. If not, download and install them from [Rust's official website](https://www.rust-lang.org/tools/install).

### 2. **Clone the Repository**  
```bash
git clone <your-repo-url>
cd world-simulator
```

### 3. **Run the Simulator**  
```bash
cargo run
```


## How to Use

- **Controls**:
  - **`P`**: Pause the simulation.
  - **`R`**: Resume the simulation when paused.
  - **`Q`**: Quit the simulation when paused.
  - **`F`**: Toggle "Follow Mouse" mode. When enabled, agents move toward the mouse position.

- **Mouse Interaction**:  
  Move the mouse around to influence agent behavior in "Follow Mouse" mode.

---

## Customization

You can customize various constants in the `main.rs` file:
- `VISION_RANGE`: Adjust the distance agents can "see."
- `SCALING_FACTOR`: Set the size of the simulation grid cells.
- `GAME_SPEED`: Change the simulation update interval (in milliseconds).
- `AGENT_DRAW_SIZE`: Modify the visual size of agents.

---

## Dependencies

- [ggez](https://ggez.rs): A lightweight game framework for Rust.
- [rayon](https://github.com/rayon-rs/rayon): For parallel processing.

---

## License

This project is licensed under the MIT License. Feel free to use and modify the code. Contributions are welcome!

---

Enjoy your simulation! 🕹️
