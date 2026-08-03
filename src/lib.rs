//! Rebellion library crate — exposes game systems for benchmarks.

#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

pub mod app_builder;
pub(crate) mod assets;
pub(crate) mod content;
pub mod core;
pub(crate) mod diagnostics;
pub mod entities;
pub mod gameplay;
pub mod games;
pub(crate) mod platform;
pub(crate) mod presentation;
pub mod replay;
pub mod simulation;
pub mod systems;
pub mod ui;
