//! Module Select

#![allow(dead_code)]

use super::common::*;
use crate::core::*;
use crate::games::ActiveModule;
use crate::systems::JoystickState;
use crate::ui::TransitionEvent;
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct ModuleSelectRoot;

/// Run condition: is the active module Elder Fleet (default)?
pub(crate) fn is_elder_fleet(active_module: Res<ActiveModule>) -> bool {
    active_module.is_elder_fleet()
}

pub(crate) fn spawn_module_select(
    mut commands: Commands,
    mut selection: ResMut<MenuSelection>,
    faction_icons: Res<crate::assets::FactionIconCache>,
    bindings: Res<KeyBindings>,
) {
    selection.index = 0;
    selection.total = 2; // The two faction-conflict chapters.

    commands
        .spawn((
            ModuleSelectRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            // Deep-space background with faint cyan undertone
            BackgroundColor(Color::srgb(0.015, 0.025, 0.045)),
        ))
        .with_children(|parent| {
            // Kicker line above title
            parent.spawn((
                Text::new("NEW EDEN"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgba(0.45, 0.70, 0.95, 0.75)),
            ));

            // Main title
            parent.spawn((
                Text::new("SELECT CHAPTER"),
                TextFont {
                    font_size: 40.0,
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.95, 1.0)),
            ));

            // Accent line under title
            parent.spawn((
                Node {
                    width: Val::Px(220.0),
                    height: Val::Px(2.0),
                    margin: UiRect::vertical(Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.3, 0.6, 1.0, 0.6)),
            ));

            parent.spawn(Node {
                height: Val::Px(12.0),
                ..default()
            });

            // 2×2 grid container — cards wrap; gaps consistent
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(24.0),
                    row_gap: Val::Px(24.0),
                    max_width: Val::Px(640.0),
                    justify_content: JustifyContent::Center,
                    ..default()
                })
                .with_children(|row| {
                    // Elder Fleet — Minmatar Republic vs Amarr Empire
                    spawn_module_card(
                        row,
                        0,
                        "MINMATAR / AMARR",
                        "The Elder Fleet",
                        "Republic or Empire.\nChoose your side of the conflict.",
                        Color::srgb(0.8, 0.5, 0.2),
                        CardIcon::FactionVs(Faction::Minmatar, Faction::Amarr),
                        &faction_icons,
                    );

                    // Caldari Prime — Caldari State vs Gallente Federation
                    spawn_module_card(
                        row,
                        1,
                        "CALDARI / GALLENTE",
                        "Battle of Caldari Prime",
                        "State or Federation.\nThree-mission campaign playtest.",
                        Color::srgb(0.2, 0.4, 0.7),
                        CardIcon::FactionVs(Faction::Caldari, Faction::Gallente),
                        &faction_icons,
                    );
                });

            parent.spawn(Node {
                height: Val::Px(30.0),
                ..default()
            });

            // Instructions
            parent.spawn((
                Text::new(menu_hint(&bindings, "Select", "Back")),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
            ));
        });
}

/// How a module card's emblem area should render.
pub(crate) enum CardIcon<'a> {
    /// Single Unicode glyph centred in the emblem box.
    Glyph(&'a str),
    /// Two faction emblems separated by "VS".
    FactionVs(Faction, Faction),
    /// A single non-empire emblem (Triglavian, Deathless) centred and large.
    SoloEmblem(&'a str),
}

fn spawn_module_card(
    parent: &mut ChildBuilder,
    index: usize,
    title: &str,
    subtitle: &str,
    description: &str,
    color: Color,
    icon: CardIcon,
    faction_icons: &crate::assets::FactionIconCache,
) {
    parent
        .spawn((
            MenuItem { index },
            Node {
                width: Val::Px(256.0),
                height: Val::Px(360.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(0.0)),
                border: UiRect::all(Val::Px(2.0)),
                row_gap: Val::Px(0.0),
                overflow: Overflow::clip(),
                ..default()
            },
            // Deep-space card background — near black, tinted faintly by faction
            BackgroundColor(Color::srgba(0.04, 0.06, 0.10, 0.95)),
            BorderColor(color.with_alpha(0.55)),
        ))
        .with_children(|card| {
            // Top accent stripe — faction colour bar, EVE's signature flair
            card.spawn((
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(5.0),
                    ..default()
                },
                BackgroundColor(color),
            ));

            // Inner content container with padding
            let content_color = color;
            card.spawn(Node {
                width: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(14.0)),
                row_gap: Val::Px(10.0),
                ..default()
            })
            .with_children(|card| {
                let _ = content_color;
                // Emblem area — either a single glyph or faction-vs-faction carriers
                card.spawn((
                    Node {
                        width: Val::Px(196.0),
                        height: Val::Px(86.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        margin: UiRect::bottom(Val::Px(6.0)),
                        column_gap: Val::Px(6.0),
                        ..default()
                    },
                    BackgroundColor(color.with_alpha(0.4)),
                    BorderColor(color),
                ))
                .with_children(|emblem| match icon {
                    CardIcon::Glyph(symbol) => {
                        emblem.spawn((
                            Text::new(symbol),
                            TextFont {
                                font_size: 48.0,
                                ..default()
                            },
                            TextColor(color),
                        ));
                    }
                    CardIcon::FactionVs(left, right) => {
                        spawn_faction_emblem(emblem, faction_icons, left);
                        emblem.spawn((
                            Text::new("VS"),
                            TextFont {
                                font_size: 18.0,
                                ..default()
                            },
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ));
                        spawn_faction_emblem(emblem, faction_icons, right);
                    }
                    CardIcon::SoloEmblem(key) => {
                        if let Some(image) = faction_icons.get_extra(key) {
                            emblem.spawn((
                                Node {
                                    width: Val::Px(84.0),
                                    height: Val::Px(84.0),
                                    ..default()
                                },
                                ImageNode { image, ..default() },
                            ));
                        } else {
                            emblem.spawn((
                                Text::new("?"),
                                TextFont {
                                    font_size: 48.0,
                                    ..default()
                                },
                                TextColor(color),
                            ));
                        }
                    }
                });

                // Title
                card.spawn((
                    Text::new(title),
                    TextFont {
                        font_size: 20.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));

                // Subtitle
                card.spawn((
                    Text::new(subtitle),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(color),
                ));

                // Description
                card.spawn((
                    Text::new(description),
                    TextFont {
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                    Node {
                        max_width: Val::Px(220.0),
                        ..default()
                    },
                ));
            }); // close inner content container
        });
}

fn spawn_faction_emblem(
    parent: &mut ChildBuilder,
    icons: &crate::assets::FactionIconCache,
    faction: Faction,
) {
    if let Some(image) = icons.get(faction) {
        parent.spawn((
            Node {
                width: Val::Px(68.0),
                height: Val::Px(68.0),
                ..default()
            },
            ImageNode { image, ..default() },
        ));
    } else {
        // Fallback — faction-tinted box if PNG missing
        let tint = faction.primary_color();
        parent.spawn((
            Node {
                width: Val::Px(58.0),
                height: Val::Px(58.0),
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            BackgroundColor(tint.with_alpha(0.5)),
            BorderColor(tint),
        ));
    }
}

pub(crate) fn module_select_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    bindings: Res<KeyBindings>,
    joystick: Res<JoystickState>,
    mut selection: ResMut<MenuSelection>,
    mut active_module: ResMut<ActiveModule>,
    mut endless: ResMut<crate::core::EndlessMode>,
    mut abyssal: ResMut<crate::games::abyssal_depths::AbyssalState>,
    mut session: ResMut<GameSession>,
    (mut campaign, mut cg_campaign, mut slice_mode, mut ef_campaign): (
        ResMut<CampaignState>,
        ResMut<crate::games::caldari_gallente::CGCampaignState>,
        ResMut<crate::games::caldari_gallente::VerticalSliceMode>,
        ResMut<crate::games::elder_fleet::ElderFleetCampaignState>,
    ),
    time: Res<Time>,
    mut transitions: EventWriter<TransitionEvent>,
    mut cards: Query<(&MenuItem, &mut BackgroundColor, &mut BorderColor)>,
) {
    selection.cooldown -= time.delta_secs();

    // Navigation
    let nav = get_nav_input(&keyboard, &joystick, &bindings);
    if nav != 0 && selection.cooldown <= 0.0 {
        selection.index =
            (selection.index as i32 + nav).rem_euclid(selection.total as i32) as usize;
        selection.cooldown = MENU_NAV_COOLDOWN;
    }

    // Update card highlights
    let colors = [
        Color::srgb(0.8, 0.5, 0.2), // Elder Fleet orange
        Color::srgb(0.2, 0.4, 0.7), // Caldari blue
        Color::srgb(0.6, 0.2, 0.6), // Abyssal purple
        Color::srgb(0.7, 0.2, 0.2), // Endless red
    ];

    // Time-based pulse for the selected card's border so it reads clearly.
    let t = time.elapsed_secs();
    let pulse = 0.65 + 0.35 * (t * 3.5).sin();

    for (item, mut bg, mut border) in cards.iter_mut() {
        let color = colors.get(item.index).copied().unwrap_or(colors[0]);
        let is_selected = item.index == selection.index;

        if is_selected {
            // Dark interior with faction tint + pulsing bright border
            let r = color.to_srgba();
            *bg = BackgroundColor(Color::srgba(
                r.red * 0.18,
                r.green * 0.18,
                r.blue * 0.18,
                0.95,
            ));
            *border = BorderColor(Color::srgba(
                (r.red * pulse).min(1.0),
                (r.green * pulse).min(1.0),
                (r.blue * pulse).min(1.0),
                1.0,
            ));
        } else {
            // Unselected — near black interior, dim border
            *bg = BackgroundColor(Color::srgba(0.04, 0.06, 0.10, 0.92));
            *border = BorderColor(color.with_alpha(0.35));
        }
    }

    // Confirm selection
    if is_confirm(&keyboard, &joystick, &bindings) {
        *campaign = CampaignState::default();
        ef_campaign.reset();
        *cg_campaign = crate::games::caldari_gallente::CGCampaignState::default();
        *slice_mode = crate::games::caldari_gallente::VerticalSliceMode::Slice;
        match selection.index {
            0 => {
                // Elder Fleet
                active_module.set_module("elder_fleet");
                endless.active = false;
                abyssal.active = false;
                session.chapter_ship_override = None;
                info!("Selected Elder Fleet campaign");
                transitions.send(TransitionEvent::to(GameState::FactionSelect));
            }
            1 => {
                // Caldari vs Gallente
                active_module.set_module("caldari_gallente");
                endless.active = false;
                abyssal.active = false;
                session.chapter_ship_override = None;
                info!("Selected Caldari vs Gallente campaign");
                transitions.send(TransitionEvent::to(GameState::FactionSelect));
            }
            _ => {}
        }
    }

    // Back to main menu
    if is_cancel(&keyboard, &joystick, &bindings) {
        transitions.send(TransitionEvent::to(GameState::MainMenu));
    }
}
