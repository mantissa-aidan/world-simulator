pub const VISION_RANGE: f32 = 100.0;
pub const CATCH_RADIUS: f32 = 1.0; // Distance at which Type A catches Type B
pub const MIN_AGENT_DISTANCE: f32 = 1.5; // Just enough to prevent overlap but allow touching
pub const SCALING_FACTOR: f32 = 20.0;
pub const AGENT_DRAW_SIZE: f32 = 4.0;
pub const MIN_GAME_SPEED: u64 = 16; // Maximum speed (~60fps)
pub const MAX_GAME_SPEED: u64 = 100; // Minimum speed (~10fps)
pub const DEFAULT_GAME_SPEED: u64 = 50; // Default speed (~20fps)
pub const WINDOW_WIDTH: f32 = 1600.0;
pub const WINDOW_HEIGHT: f32 = 900.0;
pub const WORLD_WIDTH: i32 = 200;
pub const WORLD_HEIGHT: i32 = 112;
pub const CATCH_DISTANCE: f32 = 5.0;

// Movement constants
pub const BASE_PREDATOR_SPEED: f32 = 1.5;
pub const BASE_PREY_SPEED: f32 = 2.0; // Prey moves faster than predators
pub const MAX_ROTATION: f32 = 0.05; // Smaller rotation for smoother turns
pub const SMOOTHING_FACTOR: f32 = 0.1; // For interpolating movement changes 