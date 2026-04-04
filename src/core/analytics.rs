//! Gameplay Analytics
//!
//! Tracks gameplay events for balancing insights: deaths by stage,
//! ship popularity, difficulty distribution, session duration.
//! Data stored locally in save data — no external services.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::{GameState, SaveData};

/// Analytics plugin — passively records gameplay events
pub struct AnalyticsPlugin;

impl Plugin for AnalyticsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SessionAnalytics>()
            .add_event::<AnalyticsEvent>()
            .add_systems(OnEnter(GameState::Playing), start_session)
            .add_systems(
                Update,
                record_events.run_if(on_event::<AnalyticsEvent>),
            )
            .add_systems(OnEnter(GameState::GameOver), end_session_death)
            .add_systems(OnEnter(GameState::Victory), end_session_victory)
            .add_systems(OnEnter(GameState::StageComplete), record_stage_complete);
    }
}

/// Current session tracking
#[derive(Resource, Default)]
pub struct SessionAnalytics {
    /// Which ship the player chose
    pub ship_name: String,
    /// Which faction
    pub faction: String,
    /// Which difficulty
    pub difficulty: String,
    /// Starting stage
    pub stage: u32,
    /// Session start time (seconds since app start)
    pub start_time: f64,
    /// Kills this session
    pub kills: u32,
    /// Damage taken this session
    pub damage_taken: f32,
    /// Bosses defeated this session
    pub bosses_defeated: u32,
}

/// Analytics events that systems can fire
#[derive(Event)]
#[allow(dead_code)]
pub enum AnalyticsEvent {
    /// Player started a mission
    MissionStarted {
        ship: String,
        faction: String,
        difficulty: String,
        stage: u32,
    },
    /// Enemy killed
    EnemyKilled,
    /// Boss defeated
    BossDefeated { stage: u32 },
    /// Player took damage
    DamageTaken { amount: f32 },
}

/// Persistent analytics data stored in save file
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct AnalyticsData {
    /// Deaths per stage (stage_number -> count)
    pub deaths_by_stage: HashMap<u32, u32>,
    /// Ship selection counts (ship_name -> count)
    pub ship_picks: HashMap<String, u32>,
    /// Difficulty selection counts (difficulty -> count)
    pub difficulty_picks: HashMap<String, u32>,
    /// Stages completed count (stage -> completions)
    pub stages_completed: HashMap<u32, u32>,
    /// Total play sessions
    pub total_sessions: u32,
    /// Total play time in seconds
    pub total_play_time_secs: f64,
}

fn start_session(mut session: ResMut<SessionAnalytics>, time: Res<Time>) {
    session.start_time = time.elapsed_secs_f64();
    session.kills = 0;
    session.damage_taken = 0.0;
    session.bosses_defeated = 0;
}

fn record_events(
    mut events: EventReader<AnalyticsEvent>,
    mut session: ResMut<SessionAnalytics>,
) {
    for event in events.read() {
        match event {
            AnalyticsEvent::MissionStarted {
                ship,
                faction,
                difficulty,
                stage,
            } => {
                session.ship_name = ship.clone();
                session.faction = faction.clone();
                session.difficulty = difficulty.clone();
                session.stage = *stage;
            }
            AnalyticsEvent::EnemyKilled => {
                session.kills += 1;
            }
            AnalyticsEvent::BossDefeated { .. } => {
                session.bosses_defeated += 1;
            }
            AnalyticsEvent::DamageTaken { amount } => {
                session.damage_taken += amount;
            }
        }
    }
}

fn end_session_death(
    session: Res<SessionAnalytics>,
    time: Res<Time>,
    mut save_data: ResMut<SaveData>,
) {
    let duration = time.elapsed_secs_f64() - session.start_time;
    let analytics = &mut save_data.analytics;

    // Record death location
    *analytics.deaths_by_stage.entry(session.stage).or_insert(0) += 1;

    // Record session stats
    record_session_common(analytics, &session, duration);
}

fn end_session_victory(
    session: Res<SessionAnalytics>,
    time: Res<Time>,
    mut save_data: ResMut<SaveData>,
) {
    let duration = time.elapsed_secs_f64() - session.start_time;
    record_session_common(&mut save_data.analytics, &session, duration);
}

fn record_stage_complete(
    session: Res<SessionAnalytics>,
    mut save_data: ResMut<SaveData>,
) {
    *save_data
        .analytics
        .stages_completed
        .entry(session.stage)
        .or_insert(0) += 1;
}

fn record_session_common(
    analytics: &mut AnalyticsData,
    session: &SessionAnalytics,
    duration: f64,
) {
    // Ship popularity
    if !session.ship_name.is_empty() {
        *analytics
            .ship_picks
            .entry(session.ship_name.clone())
            .or_insert(0) += 1;
    }

    // Difficulty distribution
    if !session.difficulty.is_empty() {
        *analytics
            .difficulty_picks
            .entry(session.difficulty.clone())
            .or_insert(0) += 1;
    }

    analytics.total_sessions += 1;
    analytics.total_play_time_secs += duration;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analytics_data_default_empty() {
        let data = AnalyticsData::default();
        assert!(data.deaths_by_stage.is_empty());
        assert!(data.ship_picks.is_empty());
        assert!(data.difficulty_picks.is_empty());
        assert_eq!(data.total_sessions, 0);
    }

    #[test]
    fn record_session_increments_counters() {
        let mut analytics = AnalyticsData::default();
        let session = SessionAnalytics {
            ship_name: "Rifter".into(),
            faction: "Minmatar".into(),
            difficulty: "Newbro".into(),
            stage: 3,
            ..default()
        };

        record_session_common(&mut analytics, &session, 120.0);

        assert_eq!(analytics.total_sessions, 1);
        assert_eq!(analytics.total_play_time_secs, 120.0);
        assert_eq!(analytics.ship_picks.get("Rifter"), Some(&1));
        assert_eq!(analytics.difficulty_picks.get("Newbro"), Some(&1));
    }

    #[test]
    fn record_session_accumulates() {
        let mut analytics = AnalyticsData::default();
        let session = SessionAnalytics {
            ship_name: "Rifter".into(),
            difficulty: "Newbro".into(),
            ..default()
        };

        record_session_common(&mut analytics, &session, 60.0);
        record_session_common(&mut analytics, &session, 90.0);

        assert_eq!(analytics.total_sessions, 2);
        assert_eq!(analytics.total_play_time_secs, 150.0);
        assert_eq!(analytics.ship_picks.get("Rifter"), Some(&2));
    }

    #[test]
    fn session_analytics_default() {
        let session = SessionAnalytics::default();
        assert_eq!(session.kills, 0);
        assert_eq!(session.damage_taken, 0.0);
        assert_eq!(session.bosses_defeated, 0);
    }
}
