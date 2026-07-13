//! Spatial Grid Utility
//!
//! Fast collision lookups using spatial partitioning. Detection and resolution
//! systems have been extracted into `src/simulation/` and `src/presentation/`.

use bevy::prelude::*;

// Spatial grid configuration
const CELL_SIZE: f32 = 50.0;
const GRID_WIDTH: usize = 18; // 800 / 50 + padding
const GRID_HEIGHT: usize = 16; // 700 / 50 + padding

/// Spatial grid for fast collision lookups
#[derive(Resource, Default)]
pub struct SpatialGrid {
    /// Grid cells containing enemy entity indices
    enemy_cells: Vec<Vec<(Entity, Vec2)>>,
}

impl SpatialGrid {
    pub fn new() -> Self {
        Self {
            enemy_cells: (0..GRID_WIDTH * GRID_HEIGHT)
                .map(|_| Vec::with_capacity(8))
                .collect(),
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.enemy_cells {
            cell.clear();
        }
    }

    #[inline]
    fn pos_to_cell(pos: Vec2) -> Option<usize> {
        // Convert from centered coords (-400..400, -350..350) to grid coords
        let gx = ((pos.x + crate::core::SCREEN_WIDTH / 2.0) / CELL_SIZE) as usize;
        let gy = ((pos.y + crate::core::SCREEN_HEIGHT / 2.0) / CELL_SIZE) as usize;

        if gx < GRID_WIDTH && gy < GRID_HEIGHT {
            Some(gy * GRID_WIDTH + gx)
        } else {
            None
        }
    }

    pub fn insert_enemy(&mut self, entity: Entity, pos: Vec2) {
        if let Some(idx) = Self::pos_to_cell(pos) {
            self.enemy_cells[idx].push((entity, pos));
        }
    }

    /// Get enemies in the same cell and adjacent cells (for border cases)
    pub fn get_nearby_enemies(&self, pos: Vec2) -> impl Iterator<Item = &(Entity, Vec2)> {
        let gx = ((pos.x + crate::core::SCREEN_WIDTH / 2.0) / CELL_SIZE) as i32;
        let gy = ((pos.y + crate::core::SCREEN_HEIGHT / 2.0) / CELL_SIZE) as i32;

        // Check 3x3 neighborhood for robustness
        let mut indices = Vec::with_capacity(9);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = gx + dx;
                let ny = gy + dy;
                if nx >= 0 && nx < GRID_WIDTH as i32 && ny >= 0 && ny < GRID_HEIGHT as i32 {
                    indices.push((ny * GRID_WIDTH as i32 + nx) as usize);
                }
            }
        }

        indices
            .into_iter()
            .flat_map(move |idx| self.enemy_cells[idx].iter())
    }
}

/// Collision plugin — now empty. All collision systems are registered
/// directly by SimulationPlugin and PresentationPlugin.
pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, _app: &mut App) {
        // Intentionally no-op — systems registered by SimulationPlugin.
    }
}
