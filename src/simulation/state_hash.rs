//! State Hashing
//!
//! Computes a deterministic hash of the simulation state each fixed tick.
//! Used for determinism verification and regression testing.

use bevy::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::sim_id::SimId;

/// Computed hash of the current simulation state.
/// Updated at the end of every fixed tick.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SimStateHash(pub u64);

/// Hashes a `Transform` into the provided hasher.
/// Uses `to_bits()` on all float components for deterministic hashing.
fn hash_transform<H: Hasher>(hasher: &mut H, transform: &Transform) {
    hash_vec3(hasher, transform.translation);
    hash_quat(hasher, transform.rotation);
    hash_vec3(hasher, transform.scale);
}

/// Hashes a `Vec3` by hashing each float component's bits.
fn hash_vec3<H: Hasher>(hasher: &mut H, v: Vec3) {
    v.x.to_bits().hash(hasher);
    v.y.to_bits().hash(hasher);
    v.z.to_bits().hash(hasher);
}

/// Hashes a `Quat` by hashing each float component's bits.
fn hash_quat<H: Hasher>(hasher: &mut H, q: Quat) {
    q.x.to_bits().hash(hasher);
    q.y.to_bits().hash(hasher);
    q.z.to_bits().hash(hasher);
    q.w.to_bits().hash(hasher);
}

/// Computes a deterministic hash of all simulation entities.
///
/// Entities are sorted by `SimId` before hashing to ensure consistent ordering
/// regardless of internal `Entity` allocation.
///
/// Runs in `FixedUpdate` after all simulation systems.
pub fn compute_state_hash_system(
    mut state_hash: ResMut<SimStateHash>,
    query: Query<(&SimId, &Transform)>,
) {
    let mut hasher = DefaultHasher::new();

    let mut entries: Vec<_> = query.iter().collect();
    // Deterministic ordering: sort by SimId.
    entries.sort_by_key(|(sim_id, _)| sim_id.0);

    for (sim_id, transform) in entries {
        sim_id.0.hash(&mut hasher);
        hash_transform(&mut hasher, transform);
    }

    state_hash.0 = hasher.finish();
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn setup_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<SimStateHash>();
        app
    }

    #[test]
    fn state_hash_is_deterministic() {
        let mut app = setup_app();

        app.world_mut()
            .commands()
            .spawn((SimId(0), Transform::from_xyz(1.0, 2.0, 3.0)));
        app.world_mut()
            .commands()
            .spawn((SimId(1), Transform::from_xyz(4.0, 5.0, 6.0)));
        app.world_mut()
            .commands()
            .spawn((SimId(2), Transform::from_xyz(7.0, 8.0, 9.0)));

        app.world_mut()
            .run_system_once(compute_state_hash_system)
            .expect("system runs");
        let first_hash = app.world().resource::<SimStateHash>().0;
        assert_ne!(first_hash, 0, "state hash should be non-zero");

        // Same state → same hash
        app.world_mut()
            .run_system_once(compute_state_hash_system)
            .expect("system runs");
        let second_hash = app.world().resource::<SimStateHash>().0;
        assert_eq!(
            first_hash, second_hash,
            "identical state should produce identical hash"
        );
    }

    #[test]
    fn state_hash_changes_with_transform() {
        let mut app = setup_app();

        app.world_mut()
            .spawn((SimId(0), Transform::from_xyz(1.0, 2.0, 3.0)));
        app.world_mut()
            .spawn((SimId(1), Transform::from_xyz(4.0, 5.0, 6.0)));

        app.world_mut()
            .run_system_once(compute_state_hash_system)
            .expect("system runs");
        let hash_before = app.world().resource::<SimStateHash>().0;

        // Mutate one transform
        let mut query = app.world_mut().query::<(&SimId, &mut Transform)>();
        for (sim_id, mut transform) in query.iter_mut(app.world_mut()) {
            if sim_id.0 == 1 {
                transform.translation.x += 0.1;
            }
        }

        // Verify mutation persisted
        let transforms: Vec<Vec3> = {
            let mut verify = app.world_mut().query::<&Transform>();
            verify.iter(app.world()).map(|t| t.translation).collect()
        };
        assert_eq!(transforms.len(), 2);

        app.world_mut()
            .run_system_once(compute_state_hash_system)
            .expect("system runs");
        let hash_after = app.world().resource::<SimStateHash>().0;

        assert_ne!(
            hash_before, hash_after,
            "changing a transform should change the state hash"
        );
    }

    #[test]
    fn state_hash_changes_with_entity_count() {
        let mut app = setup_app();

        app.world_mut()
            .spawn((SimId(0), Transform::from_xyz(1.0, 2.0, 3.0)));

        app.world_mut()
            .run_system_once(compute_state_hash_system)
            .expect("system runs");
        let hash_one = app.world().resource::<SimStateHash>().0;

        app.world_mut()
            .spawn((SimId(1), Transform::from_xyz(4.0, 5.0, 6.0)));

        app.world_mut()
            .run_system_once(compute_state_hash_system)
            .expect("system runs");
        let hash_two = app.world().resource::<SimStateHash>().0;

        assert_ne!(
            hash_one, hash_two,
            "adding an entity should change the state hash"
        );
    }
}
