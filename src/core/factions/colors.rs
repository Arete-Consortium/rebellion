//! Faction color definitions — primary, secondary, and engine trail colors.

use bevy::prelude::*;

use super::Faction;

/// Color methods for factions
impl Faction {
    /// Primary color (bright accent)
    pub fn primary_color(&self) -> Color {
        match self {
            Faction::Minmatar => Color::srgb(0.71, 0.39, 0.20), // Rust orange
            Faction::Amarr => Color::srgb(1.0, 0.84, 0.0),      // Gold
            Faction::Caldari => Color::srgb(0.27, 0.51, 0.71),  // Steel blue
            Faction::Gallente => Color::srgb(0.42, 0.56, 0.14), // Olive green
        }
    }

    /// Secondary color (darker)
    pub fn secondary_color(&self) -> Color {
        match self {
            Faction::Minmatar => Color::srgb(0.55, 0.35, 0.17), // Brown
            Faction::Amarr => Color::srgb(0.55, 0.46, 0.0),     // Dark gold
            Faction::Caldari => Color::srgb(0.12, 0.23, 0.37),  // Navy
            Faction::Gallente => Color::srgb(0.18, 0.36, 0.18), // Dark green
        }
    }

    /// Engine trail color
    pub fn engine_color(&self) -> Color {
        match self {
            Faction::Minmatar => Color::srgba(1.0, 0.59, 0.2, 0.9), // Orange
            Faction::Amarr => Color::srgba(0.39, 0.59, 1.0, 0.9),   // Blue
            Faction::Caldari => Color::srgba(0.39, 0.78, 1.0, 0.9), // Cyan
            Faction::Gallente => Color::srgba(0.59, 1.0, 0.59, 0.9), // Green
        }
    }
}
