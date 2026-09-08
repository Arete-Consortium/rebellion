//! The Hedion transport encounter. Campaign progression waits for the
//! objective as well as the scheduled carrier waves before the commander.

use bevy::prelude::*;

use super::{mission_info, ElderFleetCampaignState};
use crate::assets::ShipSpriteCache;
use crate::core::{GameState, LAYER_PLAYER};
use crate::entities::{EscortData, Friendly, Hitbox, Player};
use crate::games::ActiveModule;

pub const TRANSPORT_TYPE_ID: u32 = 1944; // Bestower industrial
pub const TRANSFER_RADIUS: f32 = 120.0;
pub const BOARDING_SECONDS: f32 = 12.0;
pub const ESCORT_SECONDS: f32 = 40.0;
pub const DEADLINE_SECONDS: f32 = 90.0;
pub const DEPARTURE_SECONDS: f32 = 1.5;
pub const TRANSPORT_HEALTH: f32 = 400.0;

/// Mission-specific completion contract, used by briefing and runtime alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EFMissionObjective {
    #[default]
    ClearWaves,
    LiberateTransport,
    EscortTransport,
}

impl EFMissionObjective {
    pub fn briefing(self) -> &'static str {
        match self {
            Self::ClearWaves => "Clear the enemy waves and defeat their commander.",
            Self::LiberateTransport => "Stay inside the transport's ring for 12 seconds to board. Dodge freely: boarding progress is retained. Keep the transport intact and secure it before the 90-second warp deadline, then clear the waves and defeat the commander.",
            Self::EscortTransport => "Stay inside the transport's ring to guide it to evacuation (40 seconds in range). Protect it from marked raiders. Reach safety before the 90-second deadline, then clear the waves and defeat the commander.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportFailure {
    Destroyed,
    Escaped,
}

impl TransportFailure {
    pub fn message(self) -> &'static str {
        match self {
            Self::Destroyed => "The transport was destroyed. Intercept incoming fire and eliminate its attackers.",
            Self::Escaped => "The transport missed the extraction window. Stay inside its ring to advance the objective.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TransportPhase {
    #[default]
    Inactive,
    Active,
    Secured,
    Failed(TransportFailure),
}

/// One attempt's progress; reset on launch/retry, preserved across pause and
/// results. This is transient state and does not change the save schema.
#[derive(Resource, Debug, Default)]
pub struct TransportObjective {
    pub kind: EFMissionObjective,
    pub phase: TransportPhase,
    pub elapsed: f32,
    pub work_seconds: f32,
    pub health_fraction: f32,
    pub in_range: bool,
    pub departure_seconds: f32,
}

impl TransportObjective {
    pub fn required_seconds(&self) -> f32 {
        match self.kind {
            EFMissionObjective::EscortTransport => ESCORT_SECONDS,
            _ => BOARDING_SECONDS,
        }
    }

    pub fn progress(&self) -> f32 {
        (self.work_seconds / self.required_seconds()).clamp(0.0, 1.0)
    }

    pub fn remaining_seconds(&self) -> f32 {
        (DEADLINE_SECONDS - self.elapsed).max(0.0)
    }

    pub fn ready_for_boss(&self) -> bool {
        self.phase == TransportPhase::Inactive
            || (self.phase == TransportPhase::Secured
                && self.departure_seconds >= DEPARTURE_SECONDS)
    }

    pub fn failure(&self) -> Option<TransportFailure> {
        if let TransportPhase::Failed(reason) = self.phase {
            Some(reason)
        } else {
            None
        }
    }

    pub fn success_label(&self) -> &'static str {
        match self.kind {
            EFMissionObjective::LiberateTransport => "TRANSPORT LIBERATED",
            _ => "CONVOY EVACUATED",
        }
    }

    pub fn hud_instruction(&self) -> &'static str {
        match self.phase {
            TransportPhase::Secured => {
                "Transport secured. Clear the waves and defeat the commander."
            }
            TransportPhase::Failed(_) => "Transport objective failed.",
            TransportPhase::Active if self.kind == EFMissionObjective::LiberateTransport => {
                if self.in_range {
                    "Boarding link active. Dodge and keep the transport intact."
                } else {
                    "Enter the transport ring to board. Progress is retained when dodging."
                }
            }
            TransportPhase::Active => {
                if self.in_range {
                    "Convoy moving. Destroy marked raiders and intercept fire."
                } else {
                    "Enter the transport ring to guide the convoy to evacuation."
                }
            }
            _ => "Clear the waves. Defeat the enemy commander.",
        }
    }

    pub fn hud_status(&self) -> String {
        match self.phase {
            TransportPhase::Active => format!(
                "{} {:.0}%  |  HULL {:.0}%  |  {:.0}s LEFT",
                if self.kind == EFMissionObjective::LiberateTransport {
                    "BOARDING"
                } else {
                    "EVACUATION"
                },
                self.progress() * 100.0,
                self.health_fraction * 100.0,
                self.remaining_seconds().ceil()
            ),
            TransportPhase::Secured => self.success_label().into(),
            _ => String::new(),
        }
    }
}

/// A noncombatant industrial hull; never an Enemy or a player weapon target.
#[derive(Component)]
pub struct MissionTransport;

pub fn start_transport_objective(
    mut commands: Commands,
    active: Res<ActiveModule>,
    campaign: Res<ElderFleetCampaignState>,
    mut objective: ResMut<TransportObjective>,
    sprites: Res<ShipSpriteCache>,
    existing: Query<Entity, With<MissionTransport>>,
) {
    // Defensive cleanup also covers an explicit restart from pause.
    for entity in &existing {
        commands.entity(entity).despawn_recursive();
    }
    *objective = TransportObjective::default();
    let kind = mission_info(&active, campaign.current_mission)
        .map(|mission| mission.objective)
        .unwrap_or_default();
    if kind == EFMissionObjective::ClearWaves {
        return;
    }
    *objective = TransportObjective {
        kind,
        phase: TransportPhase::Active,
        health_fraction: 1.0,
        ..default()
    };
    commands
        .spawn((
            MissionTransport,
            Friendly,
            Name::new("Hedion transport / Bestower"),
            EscortData {
                health: TRANSPORT_HEALTH,
                max_health: TRANSPORT_HEALTH,
                ..default()
            },
            Hitbox { radius: 30.0 },
            Transform::from_xyz(0.0, -100.0, LAYER_PLAYER - 1.0),
            Visibility::Visible,
        ))
        .with_children(|parent| {
            if let Some(image) = sprites.get(TRANSPORT_TYPE_ID) {
                parent.spawn((
                    Sprite {
                        image,
                        custom_size: Some(Vec2::splat(150.0)),
                        ..default()
                    },
                    Transform::from_rotation(Quat::from_rotation_z(
                        crate::entities::get_ship_rotation_correction(TRANSPORT_TYPE_ID),
                    )),
                ));
            }
            parent.spawn((
                Text2d::new("BESTOWER / TRANSPORT"),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(Color::srgb(0.65, 0.95, 1.0)),
                Transform::from_xyz(0.0, -86.0, 1.0),
            ));
        });
}

/// Runs after damage resolution on fixed ticks. Distance is measured to the
/// actual moving transport, not to a screen-space objective marker.
pub fn update_transport_objective(
    time: Res<Time>,
    mut objective: ResMut<TransportObjective>,
    mut transport: Query<(&mut Transform, &mut EscortData), With<MissionTransport>>,
    players: Query<&Transform, (With<Player>, Without<MissionTransport>)>,
    mut next: ResMut<NextState<GameState>>,
) {
    if matches!(*next, NextState::Pending(GameState::GameOver)) {
        return;
    }
    if matches!(
        objective.phase,
        TransportPhase::Inactive | TransportPhase::Failed(_)
    ) {
        return;
    }
    let Ok((mut transform, mut hull)) = transport.get_single_mut() else {
        if objective.phase == TransportPhase::Active {
            objective.phase = TransportPhase::Failed(TransportFailure::Destroyed);
            next.set(GameState::GameOver);
        }
        return;
    };
    let dt = time.delta_secs();
    if objective.phase == TransportPhase::Secured {
        objective.departure_seconds = (objective.departure_seconds + dt).min(DEPARTURE_SECONDS);
        // Warp out while combat continues; the escort can no longer absorb shots.
        transform.translation.y += 700.0 * dt;
        transform.scale = Vec3::new(0.6, 1.0 + objective.departure_seconds * 2.0, 1.0);
        return;
    }
    objective.health_fraction = hull.health_fraction();
    if hull.health <= 0.0 {
        objective.phase = TransportPhase::Failed(TransportFailure::Destroyed);
        next.set(GameState::GameOver);
        return;
    }
    objective.in_range = players.get_single().is_ok_and(|player| {
        player
            .translation
            .truncate()
            .distance_squared(transform.translation.truncate())
            <= TRANSFER_RADIUS.powi(2)
    });
    // Only the portion of a tick before the deadline is eligible for progress.
    // Successful boarding on the last eligible tick wins; destruction does not.
    let available = dt.min(objective.remaining_seconds());
    if objective.in_range {
        objective.work_seconds =
            (objective.work_seconds + available).min(objective.required_seconds());
    }
    objective.elapsed = (objective.elapsed + dt).min(DEADLINE_SECONDS);
    let route_fraction = if objective.kind == EFMissionObjective::EscortTransport {
        objective.progress()
    } else {
        objective.elapsed / DEADLINE_SECONDS
    };
    transform.translation.y = -100.0 + route_fraction * 280.0;
    if objective.progress() >= 1.0 {
        objective.phase = TransportPhase::Secured;
        hull.reached_end = true;
        info!("{}", objective.success_label());
    } else if objective.elapsed >= DEADLINE_SECONDS {
        objective.phase = TransportPhase::Failed(TransportFailure::Escaped);
        next.set(GameState::GameOver);
    }
}
