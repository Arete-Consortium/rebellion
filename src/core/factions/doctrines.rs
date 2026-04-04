//! Weapon and tank doctrine types for faction combat characteristics.

use bevy::prelude::*;

/// Weapon doctrine types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeaponDoctrine {
    Projectile, // Minmatar - autocannons, fast ROF, selectable damage
    Laser,      // Amarr - pulse/beam, instant hit, capacitor hungry
    Missile,    // Caldari - missiles, delayed hit, no tracking issues
    Hybrid,     // Gallente - blasters/rails, high damage, short range
}

impl WeaponDoctrine {
    pub fn name(&self) -> &'static str {
        match self {
            WeaponDoctrine::Projectile => "Autocannons",
            WeaponDoctrine::Laser => "Lasers",
            WeaponDoctrine::Missile => "Missiles",
            WeaponDoctrine::Hybrid => "Blasters",
        }
    }

    /// Projectile color
    pub fn bullet_color(&self) -> Color {
        match self {
            WeaponDoctrine::Projectile => Color::srgb(1.0, 0.8, 0.4), // Yellow-orange tracer
            WeaponDoctrine::Laser => Color::srgb(1.0, 0.9, 0.3),      // Golden beam
            WeaponDoctrine::Missile => Color::srgb(0.8, 0.9, 1.0),    // White-blue exhaust
            WeaponDoctrine::Hybrid => Color::srgb(0.4, 1.0, 0.6),     // Green plasma
        }
    }
}

/// Tank doctrine types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TankDoctrine {
    Shield, // Caldari - high shield, passive regen
    Armor,  // Amarr/Gallente - high armor, active repair
    Speed,  // Minmatar - low HP, high speed/evasion
}
