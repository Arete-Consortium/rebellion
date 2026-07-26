# Environment Collision Implementation Tracker

Implementation of authored environmental collision system (asteroids, wreckage, hard terrain) per the 15-phase spec.

## Progress

- [x] Repository audit — existing collision architecture understood
- [x] Components + events + pure helpers + unit tests (`src/entities/environment/mod.rs`)
- [x] Add environment methods to SpatialGrid (`insert_environment`, `get_nearby_environments`)
- [x] Player contact detection system (`detect_player_environment_contacts`)
- [x] Player separation + deflection + damage resolution (`resolve_player_environment_contacts`)
- [x] Projectile/environment detection + resolution (`detect_player_projectile_environment_hits`, `detect_enemy_projectile_environment_hits`, `resolve_projectile_environment_contacts`)
- [x] Mission cleanup integration (`cleanup_cg_entities` despawns `EnvironmentObject`)
- [x] Presentation feedback (`environment_hit_reactions`, `environment_destroyed_reactions`)
- [x] Spawn helper (`spawn_environment` in `entities/environment/mod.rs`)
- [x] Integration test (`tests/environment_collision.rs`)
- [ ] Debug visualization overlay
- [ ] Content validation (data-driven definitions, authored encounter geometry)
- [ ] Performance benchmark
- [ ] Native + WASM build verification
- [ ] Manual playtest
- [ ] Documentation update (ADL, topic notes)

## Notes

- **Bug fixed**: `resolve_player_environment_contacts` was passing `contact.penetration` as `player_radius` to `resolve_boundary_pin`, causing escape candidates to be evaluated with an inflated radius and pinning the player far from the obstacle. Fixed by querying `Hitbox` and passing `hitbox.radius`.
- **Test design**: The integration test disables player invincibility, spawns an asteroid overlapping the player, verifies separation + contact damage, fires a projectile from below the asteroid, and verifies destruction + state hash change.
