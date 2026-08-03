//! Combat Wheel HUD.
//!
//! Ported from `~/projects/eve-rebellion-hud/` (Bevy 0.14 prototype) into
//! Bevy 0.15. Replaces the egui capacitor wheel + bottom-bar meters for
//! the Triglavian Invasion campaign only; see `crate::games::triglavian_invasion`
//! for the campaign-gating run condition and adapter systems.
//!
//! Module-level porting notes (Bevy 0.14 → 0.15):
//! - `ColorMesh2dBundle` removed → `MaterialMesh2dBundle<ColorMaterial>` +
//!   explicit `Mesh2d` component for module slots.
//! - `RenderAssetUsages::default()` removed → `RenderAssetUsages::RENDER_WORLD`.
//! - `AudioBundle` removed → `AudioPlayer` + `PlaybackSettings` components.
//! - `Text2dBundle` deprecated → `Text2d` + `TextFont` + `TextColor` +
//!   `TextLayout` + `Transform` + `Visibility` separately.
//! - `JustifyText` moved from component to `TextLayout.justify` field.
//! - `Text` queries changed from `&mut Text` to `&mut Text2d` (Bevy 0.15 has
//!   both `Text` (the text content) and `Text2d` (the marker + content holder).
//! - Visual state enum renamed `IntegrityState` → `IntegrityVisual` to avoid
//!   name collision with the gameplay-side adapter resource field.
//!
//! Lifecycle: spawn/despawn is owned by `triglavian_invasion::TriglavianInvasionPlugin`,
//! which wires `spawn_*` on `OnEnter(GameState::Playing)` and `despawn_combat_wheel`
//! on `OnExit(GameState::Playing)`, gated by `in_triglavian_invasion`. This
//! plugin only registers materials, resources, and the per-update bind
//! systems (which are inert until entities exist).

pub mod audio;
pub mod components;
pub mod events;
pub mod layout;
pub mod materials;
pub mod mesh_gen;
pub mod resources;
pub mod spawn;
pub mod systems;
pub mod text;

pub use audio::*;
pub use components::*;
pub use events::*;
pub use layout::*;
pub use materials::*;
pub use mesh_gen::*;
pub use resources::*;
pub use spawn::*;
pub use systems::*;
pub use text::*;

use bevy::prelude::*;
use bevy::sprite::Material2dPlugin;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HudBindingSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct HudAnimationSet;

/// Plugin entry. Spawn/despawn is gated by the `in_triglavian_invasion` run
/// condition in `crate::games::triglavian_invasion::TriglavianInvasionPlugin::build()`.
pub struct CombatWheelPlugin;

impl Plugin for CombatWheelPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudSettings>()
            .init_resource::<AccessibilitySettings>()
            .init_resource::<FactionSkin>()
            .init_resource::<ActiveInputDevice>()
            .init_resource::<AudioManager>()
            .init_resource::<CombatWheelAdapter>()
            .add_plugins(Material2dPlugin::<ShieldMaterial>::default())
            .add_plugins(Material2dPlugin::<IntegrityMaterial>::default())
            .add_plugins(Material2dPlugin::<HeatMaterial>::default())
            .add_plugins(Material2dPlugin::<CapacitorMaterial>::default())
            .add_event::<CombatWheelEvent>()
            .add_systems(
                Update,
                (
                    emit_combat_wheel_events.before(HudBindingSet),
                    update_faction_materials.before(HudBindingSet),
                    (
                        bind_shield_to_wheel,
                        bind_integrity_to_wheel,
                        bind_capacitor_to_wheel,
                        bind_heat_to_wheel,
                        bind_modules_to_wheel,
                    )
                        .in_set(HudBindingSet),
                    (
                        trigger_shield_surge,
                        animate_shield_surge,
                        animate_capacitor_pulse,
                        animate_heat_glow,
                    )
                        .in_set(HudAnimationSet)
                        .after(HudBindingSet),
                    update_input_glyphs.after(HudBindingSet),
                    update_module_text.after(HudBindingSet),
                    update_percentage_labels.after(HudBindingSet),
                    cycle_faction_skin,
                    update_accessibility.after(HudBindingSet),
                    resolve_layout.in_set(HudBindingSet),
                    update_audio_feedback.after(HudAnimationSet),
                ),
            )
            .configure_sets(Update, (HudBindingSet, HudAnimationSet).chain());
    }
}
