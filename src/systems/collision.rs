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
    /// Grid cells containing environmental object entity indices
    environment_cells: Vec<Vec<(Entity, Vec2, f32)>>,
}

impl SpatialGrid {
    pub fn new() -> Self {
        let cell_count = GRID_WIDTH * GRID_HEIGHT;
        Self {
            enemy_cells: (0..cell_count)
                .map(|_| Vec::with_capacity(8))
                .collect(),
            environment_cells: (0..cell_count)
                .map(|_| Vec::with_capacity(4))
                .collect(),
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.enemy_cells {
            cell.clear();
        }
        for cell in &mut self.environment_cells {
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

    pub fn insert_environment(&mut self, entity: Entity, pos: Vec2, radius: f32) {
        // Insert into every cell intersecting the object's bounding box
        // so large objects are found from any overlapping query cell.
        let min_x = pos.x - radius;
        let min_y = pos.y - radius;
        let max_x = pos.x + radius;
        let max_y = pos.y + radius;

        let gx_min = ((min_x + crate::core::SCREEN_WIDTH / 2.0) / CELL_SIZE) as i32;
        let gy_min = ((min_y + crate::core::SCREEN_HEIGHT / 2.0) / CELL_SIZE) as i32;
        let gx_max = ((max_x + crate::core::SCREEN_WIDTH / 2.0) / CELL_SIZE) as i32;
        let gy_max = ((max_y + crate::core::SCREEN_HEIGHT / 2.0) / CELL_SIZE) as i32;

        let gx_min = gx_min.max(0) as usize;
        let gy_min = gy_min.max(0) as usize;
        let gx_max = (gx_max as usize).min(GRID_WIDTH - 1);
        let gy_max = (gy_max as usize).min(GRID_HEIGHT - 1);

        for gy in gy_min..=gy_max {
            for gx in gx_min..=gx_max {
                let idx = gy * GRID_WIDTH + gx;
                self.environment_cells[idx].push((entity, pos, radius));
            }
        }
    }

    /// Get environment objects in the same cell and adjacent cells.
    /// Deduplicates entities that span multiple grid cells.
    pub fn get_nearby_environments(&self, pos: Vec2) -> Vec<(Entity, Vec2, f32)> {
        let gx = ((pos.x + crate::core::SCREEN_WIDTH / 2.0) / CELL_SIZE) as i32;
        let gy = ((pos.y + crate::core::SCREEN_HEIGHT / 2.0) / CELL_SIZE) as i32;

        let mut results = Vec::with_capacity(16);
        let mut seen = Vec::with_capacity(16);

        for dy in -1..=1 {
            for dx in -1..=1 {
                let nx = gx + dx;
                let ny = gy + dy;
                if nx >= 0 && nx < GRID_WIDTH as i32 && ny >= 0 && ny < GRID_HEIGHT as i32 {
                    let idx = (ny * GRID_WIDTH as i32 + nx) as usize;
                    for &entry in &self.environment_cells[idx] {
                        let entity = entry.0;
                        if !seen.contains(&entity) {
                            seen.push(entity);
                            results.push(entry);
                        }
                    }
                }
            }
        }

        results
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
