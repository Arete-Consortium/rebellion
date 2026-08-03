use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderRef};
use bevy::sprite::Material2d;

// ---------------------------------------------------------------------------
// Shield Material
// ---------------------------------------------------------------------------

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ShieldMaterial {
    #[uniform(0)]
    pub health: f32,
    #[uniform(0)]
    pub surge_intensity: f32,
    #[uniform(0)]
    pub time: f32,
    #[uniform(0)]
    pub _pad: f32,
    #[uniform(1)]
    pub tint: Vec4,
}

impl Material2d for ShieldMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/shield_ring.wgsl".into()
    }
}

// ---------------------------------------------------------------------------
// Integrity Material
// ---------------------------------------------------------------------------

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct IntegrityMaterial {
    #[uniform(0)]
    pub health: f32,
    #[uniform(0)]
    pub armor: f32,
    #[uniform(0)]
    pub time: f32,
    #[uniform(0)]
    pub repair_active: f32,
    #[uniform(1)]
    pub tint: Vec4,
}

impl Material2d for IntegrityMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/integrity_ring.wgsl".into()
    }
}

// ---------------------------------------------------------------------------
// Heat Material
// ---------------------------------------------------------------------------

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct HeatMaterial {
    #[uniform(0)]
    pub heat_norm: f32,
    #[uniform(0)]
    pub time: f32,
    #[uniform(0)]
    pub warning_threshold: f32,
    #[uniform(0)]
    pub critical_threshold: f32,
    #[uniform(1)]
    pub tint: Vec4,
}

impl Material2d for HeatMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/heat_arc.wgsl".into()
    }
}

// ---------------------------------------------------------------------------
// Capacitor Material
// ---------------------------------------------------------------------------

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct CapacitorMaterial {
    #[uniform(0)]
    pub energy: f32,
    #[uniform(0)]
    pub time: f32,
    #[uniform(0)]
    pub _pad0: f32,
    #[uniform(0)]
    pub _pad1: f32,
    #[uniform(1)]
    pub tint: Vec4,
}

impl Material2d for CapacitorMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/capacitor_core.wgsl".into()
    }
}

// ---------------------------------------------------------------------------
// Faction Color Palettes
// ---------------------------------------------------------------------------

use super::resources::FactionSkin;

/// Per-faction color overrides for shader uniforms.
///
/// Color choices match the canonical runtime palette
/// (`src/games/triglavian_invasion/mod.rs:127-141`) plus the
/// `src/core/constants.rs` Minmatar primary.
///
/// Verified 2026-08-02: JSON `games/triglavian_invasion/config/module.json:9-30`
/// ships different EDENCOM hex codes (`#1a5a9a` etc.) than Rust
/// (`Color::srgb(0.2, 0.6, 0.9)`). The Rust values are what drives runtime
/// visuals; JSON values are documentation-only. See ADL-20260802-001.
#[derive(Debug, Clone, Copy)]
#[allow(dead_code)] // integrity_fractured/breached/critical, capacitor_dim, heat_warning/critical
                    // are read by the shader uniforms and state-tint code wired in Phase 3.
pub struct FactionPalette {
    pub shield_base: Color,
    pub integrity_pristine: Color,
    pub integrity_fractured: Color,
    pub integrity_breached: Color,
    pub integrity_critical: Color,
    pub capacitor_dim: Color,
    pub capacitor_bright: Color,
    pub heat_safe: Color,
    pub heat_warning: Color,
    pub heat_critical: Color,
}

impl FactionSkin {
    pub fn palette(&self) -> FactionPalette {
        match self {
            // Minmatar: rust orange primary (#B56333), borrowed arc-blue shield.
            // See notes/topics/eve-rebellion/faction-skin-minmatar.md.
            FactionSkin::Minmatar => FactionPalette {
                shield_base: Color::srgb(0.18, 0.55, 0.85), // Arc blue (shield only)
                integrity_pristine: Color::srgb(0.71, 0.39, 0.20), // Minmatar rust orange
                integrity_fractured: Color::srgb(0.55, 0.28, 0.12),
                integrity_breached: Color::srgb(0.45, 0.18, 0.08),
                integrity_critical: Color::srgb(0.55, 0.0, 0.0), // Tribal red
                capacitor_dim: Color::srgb(0.30, 0.15, 0.05),
                capacitor_bright: Color::srgb(0.91, 0.49, 0.16), // Amber alert
                heat_safe: Color::srgb(0.91, 0.49, 0.02),        // Amber alert
                heat_warning: Color::srgb(0.95, 0.30, 0.05),
                heat_critical: Color::srgb(1.0, 0.10, 0.0),
            },
            // EDENCOM: coalition blue (#3399E6 / 0.2, 0.6, 0.9).
            // Matches src/games/triglavian_invasion/mod.rs:127-129.
            FactionSkin::Edencom => FactionPalette {
                shield_base: Color::srgb(0.30, 0.80, 1.0), // Accent cyan-blue
                integrity_pristine: Color::srgb(0.20, 0.60, 0.90), // EDENCOM primary
                integrity_fractured: Color::srgb(0.15, 0.45, 0.70),
                integrity_breached: Color::srgb(0.10, 0.30, 0.55),
                integrity_critical: Color::srgb(0.80, 0.10, 0.05),
                capacitor_dim: Color::srgb(0.10, 0.10, 0.15),
                capacitor_bright: Color::srgb(0.30, 0.80, 1.0), // Coalition blue accent
                heat_safe: Color::srgb(0.91, 0.49, 0.02),
                heat_warning: Color::srgb(0.95, 0.30, 0.05),
                heat_critical: Color::srgb(1.0, 0.10, 0.0),
            },
            // Triglavian: crimson (#CC3333 / 0.8, 0.2, 0.2).
            // Matches src/games/triglavian_invasion/mod.rs:138-141.
            FactionSkin::Triglavian => FactionPalette {
                shield_base: Color::srgb(1.00, 0.40, 0.20), // Accent orange-red
                integrity_pristine: Color::srgb(0.80, 0.20, 0.20), // Triglavian primary
                integrity_fractured: Color::srgb(0.55, 0.10, 0.10),
                integrity_breached: Color::srgb(0.30, 0.10, 0.10),
                integrity_critical: Color::srgb(0.10, 0.10, 0.12), // Near-black secondary
                capacitor_dim: Color::srgb(0.15, 0.05, 0.10),
                capacitor_bright: Color::srgb(1.00, 0.40, 0.20), // Orange-red accent
                heat_safe: Color::srgb(0.91, 0.49, 0.02),
                heat_warning: Color::srgb(0.95, 0.30, 0.05),
                heat_critical: Color::srgb(1.0, 0.10, 0.0),
            },
            // Other empire skins kept minimal — not used in Triglavian campaign
            // but preserved for the F-key cycle.
            FactionSkin::Amarr => FactionPalette {
                shield_base: Color::srgb(0.18, 0.55, 0.85),
                integrity_pristine: Color::srgb(1.0, 0.84, 0.0), // Amarr gold
                integrity_fractured: Color::srgb(0.65, 0.55, 0.20),
                integrity_breached: Color::srgb(0.45, 0.30, 0.10),
                integrity_critical: Color::srgb(0.80, 0.10, 0.05),
                capacitor_dim: Color::srgb(0.20, 0.15, 0.05),
                capacitor_bright: Color::srgb(1.0, 0.84, 0.20),
                heat_safe: Color::srgb(0.91, 0.49, 0.02),
                heat_warning: Color::srgb(0.95, 0.30, 0.05),
                heat_critical: Color::srgb(1.0, 0.10, 0.0),
            },
            FactionSkin::Gallente => FactionPalette {
                shield_base: Color::srgb(0.18, 0.55, 0.85),
                integrity_pristine: Color::srgb(0.42, 0.56, 0.14), // Gallente olive
                integrity_fractured: Color::srgb(0.35, 0.45, 0.10),
                integrity_breached: Color::srgb(0.25, 0.32, 0.05),
                integrity_critical: Color::srgb(0.80, 0.10, 0.05),
                capacitor_dim: Color::srgb(0.10, 0.20, 0.15),
                capacitor_bright: Color::srgb(0.42, 0.80, 0.50),
                heat_safe: Color::srgb(0.91, 0.49, 0.02),
                heat_warning: Color::srgb(0.95, 0.30, 0.05),
                heat_critical: Color::srgb(1.0, 0.10, 0.0),
            },
            FactionSkin::Caldari => FactionPalette {
                shield_base: Color::srgb(0.18, 0.55, 0.85),
                integrity_pristine: Color::srgb(0.27, 0.51, 0.71), // Caldari steel blue
                integrity_fractured: Color::srgb(0.20, 0.40, 0.55),
                integrity_breached: Color::srgb(0.15, 0.28, 0.42),
                integrity_critical: Color::srgb(0.80, 0.10, 0.05),
                capacitor_dim: Color::srgb(0.05, 0.10, 0.15),
                capacitor_bright: Color::srgb(0.27, 0.65, 0.95),
                heat_safe: Color::srgb(0.91, 0.49, 0.02),
                heat_warning: Color::srgb(0.95, 0.30, 0.05),
                heat_critical: Color::srgb(1.0, 0.10, 0.0),
            },
        }
    }
}

/// Spawn helper: creates a ShieldMaterial with default values.
pub fn default_shield_material() -> ShieldMaterial {
    ShieldMaterial {
        health: 1.0,
        surge_intensity: 0.0,
        time: 0.0,
        _pad: 0.0,
        tint: Vec4::new(1.0, 1.0, 1.0, 1.0),
    }
}

/// Spawn helper: creates an IntegrityMaterial with default values.
pub fn default_integrity_material() -> IntegrityMaterial {
    IntegrityMaterial {
        health: 1.0,
        armor: 1.0,
        time: 0.0,
        repair_active: 0.0,
        tint: Vec4::new(1.0, 1.0, 1.0, 1.0),
    }
}

/// Spawn helper: creates a HeatMaterial with default values.
pub fn default_heat_material() -> HeatMaterial {
    HeatMaterial {
        heat_norm: 0.0,
        time: 0.0,
        warning_threshold: 0.6,
        critical_threshold: 0.85,
        tint: Vec4::new(1.0, 1.0, 1.0, 1.0),
    }
}

/// Spawn helper: creates a CapacitorMaterial with default values.
pub fn default_capacitor_material() -> CapacitorMaterial {
    CapacitorMaterial {
        energy: 1.0,
        time: 0.0,
        _pad0: 0.0,
        _pad1: 0.0,
        tint: Vec4::new(0.9, 0.5, 0.1, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triglavian_palette_uses_canonical_crimson() {
        let p = FactionSkin::Triglavian.palette();
        // 0.8, 0.2, 0.2 ≈ #CC3333 — matches src/games/triglavian_invasion/mod.rs:138
        // Compare in sRGB (Bevy 0.15 Color → Srgba for channel access).
        let s = p.integrity_pristine.to_srgba();
        assert!((s.red - 0.8).abs() < 0.01);
        assert!((s.green - 0.2).abs() < 0.01);
        assert!((s.blue - 0.2).abs() < 0.01);
    }

    #[test]
    fn edencom_palette_uses_canonical_blue() {
        let p = FactionSkin::Edencom.palette();
        let s = p.integrity_pristine.to_srgba();
        assert!((s.red - 0.2).abs() < 0.01);
        assert!((s.green - 0.6).abs() < 0.01);
        assert!((s.blue - 0.9).abs() < 0.01);
    }

    #[test]
    fn minmatar_palette_uses_rust_orange() {
        let p = FactionSkin::Minmatar.palette();
        // 0.71, 0.39, 0.20 ≈ #B56333 — matches core/constants.rs Minmatar primary
        let s = p.integrity_pristine.to_srgba();
        assert!((s.red - 0.71).abs() < 0.01);
        assert!((s.green - 0.39).abs() < 0.01);
        assert!((s.blue - 0.20).abs() < 0.01);
    }
}
