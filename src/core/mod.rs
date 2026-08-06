//! Core game systems and types for Rebellion
//!
//! This module contains the fundamental building blocks:
//! - Game states and transitions
//! - Shared resources (score, currency)
//! - Custom events
//! - Game constants
//! - Faction definitions
//! - Achievement system

pub mod achievements;
pub mod analytics;
pub mod campaign;
pub mod constants;
pub mod events;
pub mod factions;
pub mod game_state;
pub mod keybindings;
pub mod resources;
pub mod save;

pub use achievements::*;
pub use analytics::{AnalyticsData, AnalyticsPlugin};
pub use campaign::*;
pub use constants::*;
pub use events::*;
pub use factions::*;
pub use game_state::*;
pub use keybindings::{Action, Binding, KeyBindings, KeyBindingsPlugin, key_label};
pub use resources::*;
pub use save::*;
