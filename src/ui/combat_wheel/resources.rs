use bevy::prelude::*;

/// Player-facing HUD settings.
#[derive(Resource)]
#[allow(dead_code)]
pub struct HudSettings {
    pub scale: f32,
    pub opacity: f32,
    pub compact_mode_threshold: Vec2,
    pub show_module_labels: bool,
    pub show_damage_numbers: bool,
    pub show_numeric_percentages: bool,
}

impl Default for HudSettings {
    fn default() -> Self {
        Self {
            scale: 1.0,
            opacity: 1.0,
            compact_mode_threshold: Vec2::new(1366.0, 768.0),
            show_module_labels: true,
            show_damage_numbers: true,
            show_numeric_percentages: false,
        }
    }
}

/// Accessibility toggles.
#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct AccessibilitySettings {
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub reduced_flashing: bool,
    pub colorblind_mode: ColorblindMode,
    pub text_labels: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum ColorblindMode {
    #[default]
    None,
    Deuteranopia,
    Protanopia,
    Tritanopia,
    Achromatopsia,
}

/// Active faction skin. Does not change interaction rules.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FactionSkin {
    #[default]
    Minmatar,
    Amarr,
    Gallente,
    Caldari,
    Triglavian,
    Edencom,
}

impl FactionSkin {
    /// Cycle order used by `cycle_faction_skin`. Module-scope for testability.
    pub fn next(&self) -> Self {
        match self {
            FactionSkin::Minmatar => FactionSkin::Amarr,
            FactionSkin::Amarr => FactionSkin::Gallente,
            FactionSkin::Gallente => FactionSkin::Caldari,
            FactionSkin::Caldari => FactionSkin::Triglavian,
            FactionSkin::Triglavian => FactionSkin::Edencom,
            FactionSkin::Edencom => FactionSkin::Minmatar,
        }
    }

    /// Map a `player_faction` string from `ActiveModule` to a `FactionSkin`.
    /// Returns `None` for unknown factions so the call site can fall back to
    /// the default skin.
    ///
    /// Recognised inputs:
    /// - `"minmatar"` → `FactionSkin::Minmatar`
    /// - `"triglavian"` → `FactionSkin::Triglavian`
    /// - `"edencom"` → `FactionSkin::Edencom`
    /// - `"amarr"|"gallente"|"caldari"` → matching empire skins
    /// - anything else → `None`
    pub fn from_player_faction(faction: &str) -> Option<Self> {
        match faction {
            "minmatar" => Some(FactionSkin::Minmatar),
            "triglavian" => Some(FactionSkin::Triglavian),
            "edencom" => Some(FactionSkin::Edencom),
            "amarr" => Some(FactionSkin::Amarr),
            "gallente" => Some(FactionSkin::Gallente),
            "caldari" => Some(FactionSkin::Caldari),
            _ => None,
        }
    }
}

/// Current input device for glyph display.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActiveInputDevice {
    #[default]
    Keyboard,
    #[allow(dead_code)]
    Mouse,
    #[allow(dead_code)]
    Gamepad,
}

/// Adapter resource — projects the real game's `ShipStats` Component +
/// `ComboHeatSystem` Resource into the scalar fields the prototype's
/// `bind_*_to_wheel` systems expect.
///
/// Owned by the triglavian_invasion `combat_wheel_bind` submodule; populated
/// by `project_ship_stats_to_combat_wheel` (FixedUpdate) and read by the
/// bind systems (Update).
#[derive(Resource, Debug, Default)]
pub struct CombatWheelAdapter {
    pub shield_current: f32,
    pub shield_max: f32,
    pub shield_recharge_rate: f32,
    pub shield_collapsed: bool,
    pub last_damage_direction: Option<Vec2>,

    pub hull_current: f32,
    pub hull_max: f32,
    pub armor_current: f32,
    pub armor_max: f32,
    pub repair_active: bool,

    pub capacitor_current: f32,
    pub capacitor_max: f32,
    pub capacitor_regen_rate: f32,

    pub heat_current: f32,
    pub heat_maximum: f32,
    pub heat_warning_threshold: f32,
    pub heat_critical_threshold: f32,
    pub heat_locked: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_player_faction_maps_known_factions() {
        assert_eq!(
            FactionSkin::from_player_faction("minmatar"),
            Some(FactionSkin::Minmatar)
        );
        assert_eq!(
            FactionSkin::from_player_faction("triglavian"),
            Some(FactionSkin::Triglavian)
        );
        assert_eq!(
            FactionSkin::from_player_faction("edencom"),
            Some(FactionSkin::Edencom)
        );
        assert_eq!(
            FactionSkin::from_player_faction("amarr"),
            Some(FactionSkin::Amarr)
        );
        assert_eq!(
            FactionSkin::from_player_faction("gallente"),
            Some(FactionSkin::Gallente)
        );
        assert_eq!(
            FactionSkin::from_player_faction("caldari"),
            Some(FactionSkin::Caldari)
        );
    }

    #[test]
    fn from_player_faction_returns_none_for_unknown() {
        assert_eq!(FactionSkin::from_player_faction("unknown"), None);
        assert_eq!(FactionSkin::from_player_faction(""), None);
        assert_eq!(FactionSkin::from_player_faction("Minmatar"), None); // case-sensitive
    }

    #[test]
    fn cycle_order_covers_all_skins() {
        // Starting from any skin, cycling returns to the same skin after 6.
        let mut skin = FactionSkin::Minmatar;
        for _ in 0..6 {
            skin = skin.next();
        }
        assert_eq!(skin, FactionSkin::Minmatar);
    }

    #[test]
    fn triglavian_and_edencom_palettes_differ() {
        // Smoke test: the two faction-specific palettes must not be
        // identical (catches copy-paste bugs in the palette table).
        let t = FactionSkin::Triglavian.palette();
        let e = FactionSkin::Edencom.palette();
        assert_ne!(
            t.integrity_pristine, e.integrity_pristine,
            "Triglavian and EDENCOM must have distinct primary colors"
        );
    }
}
