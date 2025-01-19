//! # Spatial Grid System
//! 
//! This module implements spatial partitioning for efficient agent queries.
//! It provides a grid-based system for quick neighbor lookups and range queries.
//!
//! ## Design Pattern: Spatial Partitioning
//! The `SpatialGrid` implements a grid-based spatial partitioning system:
//! - Divides the world into cells for O(1) neighbor lookups
//! - Reduces collision detection complexity from O(n²) to O(n)
//! - Enables efficient range queries for agent vision
//!
//! ## Implementation Details
//!
//! ### Spatial Grid
//! ```text
//! +---+---+---+
//! | 0 | 1 | 2 |  Each cell contains references (indices)
//! +---+---+---+  to agents within its boundaries
//! | 3 | 4 | 5 |
//! +---+---+---+  Neighbor queries check surrounding
//! | 6 | 7 | 8 |  cells for potential interactions
//! +---+---+---+
//! ```

use crate::constants::SCALING_FACTOR;
use std::f32::consts::PI;

/// A spatial partitioning grid for efficient neighbor lookups
#[derive(Clone)]
pub struct SpatialGrid {
    /// Cells containing agent indices
    cells: Vec<Vec<usize>>,
    /// Size of each cell (typically vision range)
    cell_size: f32,
    /// Number of cells horizontally
    width: usize,
    /// Number of cells vertically
    height: usize,
}

impl SpatialGrid {
    /// Creates a new spatial grid
    ///
    /// # Arguments
    /// * `width` - World width in pixels
    /// * `height` - World height in pixels
    /// * `cell_size` - Size of each cell (typically vision range)
    pub fn new(width: f32, height: f32, cell_size: f32) -> Self {
        let w = (width / cell_size).ceil() as usize;
        let h = (height / cell_size).ceil() as usize;
        let cells = vec![Vec::new(); w * h];
        SpatialGrid {
            cells,
            cell_size,
            width: w,
            height: h,
        }
    }

    /// Clears all cells in the grid
    pub fn clear(&mut self) {
        for cell in &mut self.cells {
            cell.clear();
        }
    }

    /// Gets the cell index for a given world position
    fn get_cell_index(&self, x: f32, y: f32) -> Option<usize> {
        let cell_x = (x / self.cell_size).floor() as usize;
        let cell_y = (y / self.cell_size).floor() as usize;
        
        if cell_x < self.width && cell_y < self.height {
            Some(cell_y * self.width + cell_x)
        } else {
            None
        }
    }

    /// Inserts an agent index at the given world position
    pub fn insert(&mut self, pos: (f32, f32), agent_idx: usize) {
        if let Some(idx) = self.get_cell_index(pos.0, pos.1) {
            self.cells[idx].push(agent_idx);
        }
    }

    /// Gets all agent indices within range of a position
    pub fn get_nearby(&self, pos: (f32, f32), range: f32) -> Vec<usize> {
        let mut nearby = Vec::new();
        let cell_range = (range / self.cell_size).ceil() as i32;
        let center_x = (pos.0 / self.cell_size).floor() as i32;
        let center_y = (pos.1 / self.cell_size).floor() as i32;

        for dy in -cell_range..=cell_range {
            for dx in -cell_range..=cell_range {
                let cell_x = center_x + dx;
                let cell_y = center_y + dy;
                
                if cell_x >= 0 && cell_x < self.width as i32 &&
                   cell_y >= 0 && cell_y < self.height as i32 
                {
                    let idx = (cell_y as usize) * self.width + (cell_x as usize);
                    nearby.extend(&self.cells[idx]);
                }
            }
        }

        nearby
    }

    /// Calculates the squared distance between two points
    pub fn distance_squared(pos1: (f32, f32), pos2: (f32, f32)) -> f32 {
        let dx = pos2.0 - pos1.0;
        let dy = pos2.1 - pos1.1;
        dx * dx + dy * dy
    }

    /// Calculates the distance between two points
    pub fn distance(pos1: (f32, f32), pos2: (f32, f32)) -> f32 {
        Self::distance_squared(pos1, pos2).sqrt()
    }

    /// Checks if a point is within a given range of another point
    pub fn is_within_range(pos1: (f32, f32), pos2: (f32, f32), range: f32) -> bool {
        Self::distance_squared(pos1, pos2) <= range * range
    }

    /// Converts a game position to world space position
    pub fn to_world_space(pos: (f32, f32)) -> (f32, f32) {
        (pos.0 * SCALING_FACTOR, pos.1 * SCALING_FACTOR)
    }

    /// Calculates the angle between two points
    pub fn angle_between(from: (f32, f32), to: (f32, f32)) -> f32 {
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        dy.atan2(dx)
    }

    /// Calculates the minimum angle difference between two angles
    pub fn angle_diff(angle1: f32, angle2: f32) -> f32 {
        let mut diff = (angle1 - angle2).abs();
        diff = diff.rem_euclid(2.0 * PI);
        if diff > PI {
            diff = 2.0 * PI - diff;
        }
        diff
    }

    /// Checks if an angle is within a field of view
    pub fn is_within_fov(from_pos: (f32, f32), to_pos: (f32, f32), current_angle: f32, fov: f32) -> bool {
        let angle_to_target = Self::angle_between(from_pos, to_pos);
        Self::angle_diff(angle_to_target, current_angle) <= fov / 2.0
    }
} 