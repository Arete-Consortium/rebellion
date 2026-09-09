//! Stored doses and the current Y action, using the approved pickup artwork.
use crate::assets::PowerupIconCache;
use crate::entities::{
    boosters::{BoosterInventory, BoosterKind},
    Player, PowerupEffects,
};
use bevy::prelude::*;

#[derive(Component)]
pub(super) struct BoosterSlot(BoosterKind);
#[derive(Component)]
pub(super) struct BoosterCount(BoosterKind);
#[derive(Component)]
pub(super) struct BoosterName;
#[derive(Component)]
pub(super) struct BoosterAction;

pub(super) fn spawn_booster_hud(mut commands: Commands, icons: Option<Res<PowerupIconCache>>) {
    commands
        .spawn((
            super::common::HudRoot,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                bottom: Val::Px(110.0),
                width: Val::Px(188.0),
                padding: UiRect::all(Val::Px(8.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.9)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("BOOSTERS  D-pad up/down"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.7, 0.78)),
            ));
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    for kind in BoosterKind::ALL {
                        row.spawn((
                            BoosterSlot(kind),
                            Node {
                                width: Val::Px(52.0),
                                padding: UiRect::all(Val::Px(3.0)),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                border: UiRect::all(Val::Px(1.0)),
                                ..default()
                            },
                            BorderColor(Color::NONE),
                        ))
                        .with_children(|slot| {
                            if let Some(icon) =
                                icons.as_deref().and_then(|icons| icons.get(&kind.pickup()))
                            {
                                slot.spawn((
                                    ImageNode::new(icon),
                                    Node {
                                        width: Val::Px(32.0),
                                        height: Val::Px(32.0),
                                        ..default()
                                    },
                                ));
                            }
                            slot.spawn((
                                BoosterCount(kind),
                                Text::new("0/3"),
                                TextFont {
                                    font_size: 11.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                    }
                });
            panel.spawn((
                BoosterName,
                Text::new(""),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
            panel.spawn((
                BoosterAction,
                Text::new("Collect timed boosters"),
                TextFont {
                    font_size: 10.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.85, 1.0)),
            ));
        });
}

pub(super) fn update_booster_hud(
    inventory: Res<BoosterInventory>,
    player: Query<&PowerupEffects, With<Player>>,
    mut slots: Query<(&BoosterSlot, &mut BorderColor)>,
    mut counts: Query<
        (&BoosterCount, &mut Text, &mut TextColor),
        (Without<BoosterName>, Without<BoosterAction>),
    >,
    mut names: Query<
        &mut Text,
        (
            With<BoosterName>,
            Without<BoosterCount>,
            Without<BoosterAction>,
        ),
    >,
    mut actions: Query<
        &mut Text,
        (
            With<BoosterAction>,
            Without<BoosterCount>,
            Without<BoosterName>,
        ),
    >,
) {
    let Ok(effects) = player.get_single() else {
        return;
    };
    let selected = inventory.selected();
    for (slot, mut border) in &mut slots {
        border.0 = if slot.0 == selected {
            Color::srgb(0.6, 0.85, 1.0)
        } else {
            Color::NONE
        };
    }
    for (slot, mut text, mut color) in &mut counts {
        **text = format!(
            "{}/{}",
            inventory.count(slot.0),
            BoosterInventory::CAPACITY_PER_KIND
        );
        color.0 = if inventory.count(slot.0) > 0 {
            Color::WHITE
        } else {
            Color::srgb(0.4, 0.45, 0.5)
        };
    }
    for mut text in &mut names {
        **text = selected.name().into();
    }
    for mut text in &mut actions {
        let remaining = selected.remaining(effects);
        let action = if remaining > 0.0 {
            format!("ACTIVE {:.1}s", remaining)
        } else if inventory.count(selected) > 0 {
            "Y Use dose".into()
        } else {
            "EMPTY".into()
        };
        **text = format!("{}\n{}", selected.description(), action);
    }
}
