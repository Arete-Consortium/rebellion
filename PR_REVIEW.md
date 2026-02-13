# PR #17 Review: Combat & Visual Improvements + AI Spatial Awareness

## Summary

This PR contains 3 commits (+745/-112 across 15 files) adding:
1. **Enemy spatial awareness & predictive AI** (dodge, separation, edge avoidance, leader-escort tactics)
2. **Predictive aiming** for enemy shooting (lead shots based on player velocity)
3. **Drone/Wingman projectile dodging** (allied units evade enemy fire)
4. **Disintegrator beam visuals** (rendered beam line from Triglavian enemies to player)
5. **Drone HUD status indicator** (active count + remaining lifetime)
6. **Enemy bullet trails** (BulletTrail component on enemy projectiles)
7. **Abyssal Depths room-specific spawning** (distinct enemy compositions per room)
8. **Late-game wave variety** (stages 10-13 get broader ship/behavior pools)
9. **CI caching** (Swatinem/rust-cache@v2 across all workflow jobs)
10. **Repo cleanup** (.gitignore, CONTRIBUTING.md, README.md, Cargo.toml metadata)

---

## Issues Found

### Bug: `dodge_impulse` is never reset between frames

**File:** `src/entities/enemy.rs:476`

`enemy_spatial_awareness` writes to `ai.dodge_impulse` every frame, but it does so by computing a fresh `impulse` and assigning it. This is actually correct on closer inspection -- the value is fully overwritten each frame, not accumulated. However, note that the impulse is only computed for enemies with `sensitivity > 0.0`. For enemies with `sensitivity == 0.0` (Kamikaze), the impulse variable stays at `Vec2::ZERO` but the assignment on line 476 still occurs because it's outside the `if sensitivity > 0.0` block. **This is correct behavior** -- Kamikaze enemies still get leader cohesion applied (section 4 is outside the sensitivity gate), which seems intentional. No bug here on second analysis.

### Bug: Missing bottom-edge avoidance in `enemy_spatial_awareness`

**File:** `src/entities/enemy.rs:438-447`

Edge avoidance checks left, right, and top edges but **not the bottom edge** (`pos.y < -half_h + margin`). This means enemies dodging downward or pushed by separation forces can drift below the screen bottom without resistance. Enemies already have a bounds check system (`enemy_bounds_check`) that despawns them when far off-screen, but a soft push at the bottom edge would prevent enemies from visually drifting off the bottom during dodge maneuvers.

```rust
// Missing: bottom edge avoidance
if pos.y < -half_h + EDGE_AVOIDANCE_MARGIN {
    impulse.y += (1.0 - (pos.y + half_h) / EDGE_AVOIDANCE_MARGIN) * EDGE_PUSH_STRENGTH;
}
```

**Severity:** Low -- enemies naturally move downward in this top-down shooter, so the bottom edge case may be intentionally omitted to allow enemies to exit the bottom. But if enemies are meant to stay on-screen during dodge maneuvers, this should be added.

### Performance: O(n*m) beam visual lookup

**File:** `src/systems/effects.rs:1186-1198`

`update_disintegrator_beams` iterates through all beam visuals for each enemy to find matching beams. With many Triglavian enemies this becomes O(enemies * beams). Consider using a `HashMap<Entity, Entity>` for beam-to-source mapping, or storing the beam entity on the `DisintegratorRamp` component.

**Severity:** Low -- in practice there are few Triglavian enemies simultaneously active, so this won't be a real bottleneck.

### Performance: O(n^2) enemy separation

**File:** `src/entities/enemy.rs:423-436`

`enemy_spatial_awareness` compares every enemy against every other enemy for separation, which is O(n^2). With the maximum enemy count of ~12-15 per wave, this is fine. Just noting it for awareness if enemy counts ever increase significantly.

**Severity:** Low -- acceptable for current enemy counts.

### Code duplication: Dodge logic in drone/wingman/enemy

**Files:**
- `src/entities/drone.rs:232-248` (drone dodge)
- `src/entities/wingman.rs:265-282` (wingman dodge)
- `src/entities/enemy.rs:404-420` (enemy dodge)

All three use nearly identical projectile dodge logic (detect nearby projectile, compute perpendicular escape vector, scale by urgency). The only differences are magic numbers (detection radius: 70/80/100, strength: 100/120/150). Consider extracting a shared `compute_projectile_dodge(position, projectiles, radius, strength) -> Vec2` utility function.

**Severity:** Low -- cosmetic/maintenance concern, not a bug.

### Magic numbers scattered across dodge system

**Files:** `src/entities/drone.rs`, `src/entities/wingman.rs`, `src/entities/enemy.rs`

The enemy spatial awareness system properly uses named constants (`DODGE_DETECTION_RADIUS`, `DODGE_STRENGTH`, etc.), but the drone and wingman dodge systems use inline magic numbers (70.0, 80.0, 100.0, 120.0, 0.2 approach threshold). These should either use shared constants or at minimum be local `const` declarations for readability.

**Severity:** Low -- readability/maintenance.

### `PlayerTracker` velocity spike on first frame

**File:** `src/entities/enemy.rs:357-365`

On the first frame after the player spawns, `tracker.initialized` is `false`, so velocity is not computed. On the second frame, `prev_position` equals the first frame's position, and velocity is computed normally. This is correct. However, if the player entity is despawned and respawned (e.g., on game restart), the tracker retains stale `prev_position` from the previous game, causing a velocity spike on the first frame of the new game. The tracker should be reset when entering `GameState::Playing` or similar.

**Severity:** Medium -- could cause enemies to "pre-aim" wildly for one frame on game restart.

### `leader_cohesion` applied outside sensitivity gate

**File:** `src/entities/enemy.rs:450-472`

The leader cohesion (section 4) is applied even for enemies with `dodge_sensitivity() == 0.0` (Kamikaze). This means Kamikaze enemies, which are supposed to bee-line toward the player, will be pulled toward Tank/Spawner leaders instead. This likely conflicts with Kamikaze's intended behavior of charging straight at the player.

**Severity:** Medium -- Kamikaze enemies may behave unexpectedly when near Tank/Spawner allies.

---

## Positive Observations

- **No `.unwrap()` in game logic** -- all fallible operations use `unwrap_or`, `unwrap_or_default`, or pattern matching. Follows project conventions.
- **System ordering is correct** -- the chained pipeline `update_player_tracker → enemy_spatial_awareness → enemy_movement → enemy_shooting` ensures data flows correctly through the frame.
- **Predictive aiming is well-designed** -- the `aim_accuracy` per-behavior-type approach gives distinct personality to each enemy type. Snipers lead shots precisely while linear enemies barely lead at all.
- **Disintegrator beam cleanup** is properly added to `cleanup_buff_visuals` to prevent leaked entities on state transition.
- **BulletTrail added to enemy projectiles** -- good visual consistency with player projectiles.
- **CI caching standardized** across all workflow jobs using `Swatinem/rust-cache@v2`, replacing the verbose manual cache config in pages.yml.
- **Abyssal room spawning** is now room-specific with escalating difficulty, much better than the previous uniform spawning.
- **CONTRIBUTING.md** updated to reflect actual project structure accurately.

---

## Verdict

The PR is a solid improvement to enemy AI intelligence, visual polish, and code organization. The main concerns are:
1. **PlayerTracker stale velocity on game restart** (medium) -- should reset on state transition
2. **Kamikaze pulled toward leaders** (medium) -- conflicts with intended charge behavior
3. **Missing bottom-edge avoidance** (low) -- may be intentional

Recommend merging after addressing items 1 and 2.
