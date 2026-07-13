//! SimId — Stable deterministic identifiers for entities.
//!
//! `SimId` is assigned at spawn and never changes. It is used for replay
//! serialization, NOT for runtime `Entity` references (Bevy's `Entity` is
//! still the runtime handle).
//!
//! # Determinism
//! Same spawn order → same SimIds when the `MissionSeed` is identical.

use bevy::prelude::*;

/// Stable deterministic ID assigned to every simulation entity.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SimId(pub u64);

/// Generator that produces deterministic SimIds.
///
/// # Invariants
/// - Incremented once per spawn assignment.
/// - Reset at the start of each mission.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SimIdGenerator {
    next_id: u64,
}

impl SimIdGenerator {
    /// Create a new generator starting at 0.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset the generator (call at mission start).
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.next_id = 0;
    }

    /// Generate the next SimId.
    pub fn generate(&mut self) -> SimId {
        let id = self.next_id;
        self.next_id += 1;
        SimId(id)
    }
}

/// Assign SimId to newly-spawned entities that don't already have one.
/// Runs in FixedUpdate after spawn systems.
pub fn assign_sim_ids(
    mut commands: Commands,
    mut generator: ResMut<SimIdGenerator>,
    query: Query<Entity, Without<SimId>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(generator.generate());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generator_produces_sequential_ids() {
        let mut gen = SimIdGenerator::new();
        assert_eq!(gen.generate().0, 0);
        assert_eq!(gen.generate().0, 1);
        assert_eq!(gen.generate().0, 2);
    }

    #[test]
    fn generator_reset_works() {
        let mut gen = SimIdGenerator::new();
        gen.generate();
        gen.generate();
        gen.reset();
        assert_eq!(gen.generate().0, 0);
    }
}
