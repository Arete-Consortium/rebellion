use bevy::prelude::*;

use super::components::*;

/// Marker for module text labels.
#[derive(Component)]
pub struct ModuleLabel;

/// Marker for module input key glyphs.
#[derive(Component)]
pub struct ModuleKeyGlyph;

/// Spawns text labels for all module slots.
///
/// Bevy 0.15: `Text2dBundle` is deprecated in favor of placing `Text` +
/// `Text2d` + `TextLayout` + `Transform` + `Visibility` as separate components.
/// `JustifyText` moved from a component to a field on `TextLayout`.
pub fn spawn_module_text(
    mut commands: Commands,
    modules: Query<(Entity, &ModuleSlot, &Transform), Without<ModuleLabel>>,
    wheel_query: Query<Entity, With<CombatWheel>>,
) {
    let Ok(_wheel) = wheel_query.get_single() else {
        return;
    };

    for (entity, slot, _transform) in modules.iter() {
        let label_text = match slot.slot_id {
            ModuleSlotId::PrimaryWeapon => "WEP",
            ModuleSlotId::SecondaryWeapon => "ALT",
            ModuleSlotId::Propulsion => "ENG",
            ModuleSlotId::Defense => "DEF",
            ModuleSlotId::Ability => "ABI",
            ModuleSlotId::Deployable => "DEP",
        };

        let label = commands
            .spawn((
                ModuleLabel,
                Text2d::new(label_text),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.8, 0.7)),
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                Transform::from_xyz(0.0, -24.0, 102.0),
                Visibility::default(),
            ))
            .id();

        let key_glyph = commands
            .spawn((
                ModuleKeyGlyph,
                Text2d::new("?"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout {
                    justify: JustifyText::Center,
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 102.0),
                Visibility::default(),
            ))
            .id();

        commands.entity(entity).add_child(label);
        commands.entity(entity).add_child(key_glyph);
    }
}

/// Updates key glyph text based on current input bindings.
pub fn update_module_text(
    glyphs: Query<(Entity, &Parent), With<ModuleKeyGlyph>>,
    parents: Query<&ModuleInputGlyph>,
    mut texts: Query<&mut Text2d>,
) {
    for (child_entity, parent) in glyphs.iter() {
        let Ok(glyph) = parents.get(parent.get()) else {
            continue;
        };
        let Ok(mut text) = texts.get_mut(child_entity) else {
            continue;
        };

        // Bevy 0.15: `format!("{:?}", key)` for KeyCode variants in the 0.15
        // prelude changed representation. We use explicit matching for the
        // digits to avoid relying on Debug formatting.
        let key_str = match &glyph.binding {
            InputBinding::Keyboard(key) => keycode_to_str(*key),
            InputBinding::Mouse(btn) => format!("{:?}", btn),
            InputBinding::Gamepad(btn) => format!("{:?}", btn),
            InputBinding::Unbound => "?".to_string(),
        };

        // Bevy 0.15: Text2d derefs to Text; assign via the inner field to
        // replace the section content.
        **text = key_str;
    }
}

fn keycode_to_str(key: KeyCode) -> String {
    match key {
        KeyCode::Digit1 => "1".into(),
        KeyCode::Digit2 => "2".into(),
        KeyCode::Digit3 => "3".into(),
        KeyCode::Digit4 => "4".into(),
        KeyCode::Digit5 => "5".into(),
        KeyCode::Digit6 => "6".into(),
        KeyCode::Digit7 => "7".into(),
        KeyCode::Digit8 => "8".into(),
        KeyCode::Digit9 => "9".into(),
        KeyCode::Digit0 => "0".into(),
        other => format!("{:?}", other),
    }
}
