# Mission 2 — Determinism & Foundation: PR Checklist

**Mission:** Make the game deterministic, frame-rate-independent, and testable. No gameplay behavior change.  
**Baseline commit:** `b3e40b1` (post-Mission 1)  
**Branch prefix:** `arch/m02-*`  
**Governing docs:** `MISSION_2_CANDIDATES.md`, `SIMULATION_CONTRACT.md`, `DEPENDENCY_RULES.md`

---

## 1. Branch Strategy

```
main (protected)
  └── arch/m02-fixed-update        ← PR #1: Time<Fixed> + SimSet
        └── arch/m02-sim-fixed       ← PR #2: Simulation systems
              └── arch/m02-game-fixed    ← PR #3: Gameplay systems
                    └── arch/m02-rng         ← PR #4: SimulationRng
                          └── arch/m02-pres-rng    ← PR #5: PresentationRng
                                └── arch/m02-contact     ← PR #6: Contact event split
                                      └── arch/m02-scoring   ← PR #7: Unified scoring
                                            └── arch/m02-plugins   ← PR #8: Self-registration
                                                  └── arch/m02-headless  ← PR #9: Headless constructor
                                                        └── arch/m02-simid     ← PR #10: SimId
                                                              └── arch/m02-replay    ← PR #11: Replay recording
                                                                    └── arch/m02-state-hash  ← PR #12: State hashing
```

**Rules:**
- Each branch is a child of the previous merged branch.
- No force-push to shared branches.
- Every PR must compile and pass the Milestone 2 acceptance gate before review.
- If a PR would change observable gameplay behavior (e.g., fire rate feels different), that is a regression — fix it.

---

## 2. Acceptance Gate (Every PR)

```bash
# Must pass before opening PR
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace

# For PRs touching FixedUpdate:
# cargo test --workspace --features headless
```

**Note on Clippy:** The baseline has global `#![allow(clippy::type_complexity)]` and `#![allow(clippy::too_many_arguments)]` in `main.rs` and `lib.rs`. For Mission 2, **do not remove these suppressions** — doing so would create a massive diff unrelated to determinism. Remove them in a dedicated cleanup milestone (Milestone 45).

**Additional gate for Mission 2:**
- [ ] `cargo test --workspace` passes **and** a new headless test app launches and exits cleanly (once PR #9 lands).

---

## 3. PR-by-PR Breakdown

### PR #1 — `arch/m02-fixed-update`: Configure `Time<Fixed>` and `SimSet`

**New files:**
```
src/simulation/fixed_step.rs    // Time<Fixed> configuration, SimSet definition
```

**Modified:**
- `src/main.rs` — insert `Time<Fixed>` resource, configure timestep
- `src/simulation/mod.rs` — register `SimSet::Simulation` system set
- `src/gameplay/mod.rs` — register `SimSet::Gameplay` system set
- `src/presentation/mod.rs` — register `SimSet::Presentation` system set

**Invariants:**
- `Time<Fixed>` timestep matches the existing effective framerate (60 Hz = `1.0 / 60.0`).
- All systems still run in `Update` for this PR; the `SimSet` definitions are added but unused.
- `Time<Virtual>` is NOT used — we want real fixed step, not slow-mo.

**Verification:**
- [ ] `cargo test` passes (277 tests)
- [ ] `cargo build` passes
- [ ] Manual: game launches and plays identically (no visible change yet)
- [ ] `grep -rn "SimSet" src/` shows definitions in all three plugins

**Rollback:** Revert `Time<Fixed>` insertion; remove SimSet definitions.

---

### PR #2 — `arch/m02-sim-fixed`: Migrate simulation systems to `FixedUpdate`

**Systems to move:**
- `update_spatial_grid`
- `detect_player_projectile_hits`
- `detect_enemy_projectile_hits`
- `resolve_player_projectile_damage`
- `resolve_enemy_projectile_damage`
- `resolve_enemy_deaths`
- `ProjectilePlugin` systems (projectile movement/integration)

**Invariants:**
- Systems that previously read `time.delta_secs()` now read `time.delta_secs()` from `Time<Fixed>` (identical value at 60 Hz, but stable under lag).
- Detection still runs once per fixed tick, not once per frame.
- Presentation systems remain in `Update` — they interpolate visually.

**Verification:**
- [ ] `cargo test` passes
- [ ] Manual: gameplay feels identical at 60 FPS
- [ ] Manual: gameplay feels identical at 30 FPS (lag shouldn't change simulation)
- [ ] `grep -rn "FixedUpdate" src/simulation/` returns matches

**Rollback:** Move systems back to `Update`.

---

### PR #3 — `arch/m02-game-fixed`: Migrate gameplay systems to `FixedUpdate`

**Systems to move:**
- `PlayerPlugin` movement, weapon firing cooldowns
- `EnemyPlugin` AI, pathing, shooting cooldowns
- `WingmanPlugin`, `DronePlugin`
- `AbilityPlugin` cooldowns
- `SpawningPlugin` spawn timers
- `BossPlugin` phase timers
- `ManeuverPlugin` barrel roll durations
- `CampaignPlugin` progression timers
- `combat_outcomes.rs` systems (score, salt miner, drops)
- `combat_reactions.rs` systems that use `Local<f32>` cooldowns (e.g., `last_callout`)

**Invariants:**
- Cooldowns and timers that previously used `time.delta_secs()` now use `Time<Fixed>`.
- UI/menus remain in `Update`.
- Presentation reaction systems that read events from simulation/gameplay can stay in `Update` — they consume the same events regardless of which set produced them.

**Verification:**
- [ ] `cargo test` passes
- [ ] Manual: weapon fire rate, enemy spawn rate, ability cooldowns identical
- [ ] Manual: 30 FPS and 60 FPS produce same score after 60 seconds of gameplay

**Rollback:** Move systems back to `Update`.

---

### PR #4 — `arch/m02-rng`: Introduce `SimulationRng` and `MissionSeed`

**New files:**
```
src/simulation/rng.rs           // SimulationRng resource, MissionSeed resource
```

**Modified:**
- `src/simulation/mod.rs` — register `SimulationRng` resource
- `src/simulation/resolve_damage.rs` — replace `fastrand::f32()` crit roll with `sim_rng.f32()`
- `src/gameplay/combat_outcomes.rs` — replace `fastrand::f32()` drop chance with `sim_rng.f32()`
- `src/entities/enemy/systems.rs` — replace any `fastrand` usage with `sim_rng` where it affects gameplay

**New events/resources:**
- `MissionSeed` resource (u64) — set from menu or campaign select, persisted in save.
- `SimulationRng` resource — wraps a seeded RNG, implements `Send + Sync`.

**Invariants:**
- Default `MissionSeed` is hardcoded (e.g., `42`) so tests are deterministic.
- `SimulationRng` is ONLY accessed by systems in `FixedUpdate`.
- Same seed produces same combat outcome (same crits, same drops) across runs.
- No observable gameplay change other than determinism.

**Verification:**
- [ ] `cargo test` passes
- [ ] New test: run combat scenario twice with same seed, assert identical damage results.
- [ ] `grep -rn "fastrand::f32()" src/simulation/ src/gameplay/` returns nothing

**Rollback:** Revert to `fastrand`; remove `SimulationRng` resource.

---

### PR #5 — `arch/m02-pres-rng`: `PresentationRng` for cosmetic randomness

**New files:**
```
src/presentation/rng.rs           // PresentationRng resource
```

**Modified:**
- `src/presentation/mod.rs` — register `PresentationRng` resource
- `src/presentation/combat_reactions.rs` — chain bolt jitter uses `pres_rng.f32()`
- `src/systems/effects/*.rs` — particle variance uses `pres_rng.f32()`
- `src/systems/collision.rs` — any remaining cosmetic RNG moved to `PresentationRng`

**Invariants:**
- `PresentationRng` is unseeded (or seeded from wall-clock) — cosmetic jitter does not affect gameplay.
- `PresentationRng` is ONLY accessed by systems in `Update`.
- Gameplay outcomes (damage, score, drops) are NOT affected by `PresentationRng`.

**Verification:**
- [ ] `cargo test` passes
- [ ] Same seed produces same score but different particle jitter across runs.

**Rollback:** Revert cosmetic RNG to `fastrand`.

---

### PR #6 — `arch/m02-contact`: Split `ContactDetected` into raw + resolved

**Modified:**
- `src/core/events.rs` — add `ContactRaw` event (detection only: projectile, enemy, positions)
- `src/core/events.rs` — keep `ContactDetected` as resolved event (damage, crit, ammo, etc.)
- `src/simulation/detect_collisions.rs` — emit `ContactRaw`
- `src/simulation/resolve_damage.rs` — consume `ContactRaw`, emit `ContactDetected`

**Invariants:**
- Detection systems do NOT know about damage values, crit chance, or ammo type.
- Resolution systems consume `ContactRaw` and enrich it into `ContactDetected`.
- No gameplay behavior change — event pipeline just adds one hop.

**Verification:**
- [ ] `cargo test` passes
- [ ] `grep -rn "ContactType::PlayerProjectileEnemy" src/simulation/detect_collisions.rs` returns nothing
- [ ] `grep -rn "ContactType::PlayerProjectileEnemy" src/simulation/resolve_damage.rs` returns match

**Rollback:** Revert to single `ContactDetected` emitted from detection.

---

### PR #7 — `arch/m02-scoring`: Unify `ScoringPlugin` + `ScoringSystemPlugin`

**Modified:**
- `src/systems/scoring.rs` — audit which systems from here are still needed
- `src/systems/scoring_v2.rs` — audit which systems from here are still needed
- `src/gameplay/mod.rs` — replace dual registration with single `UnifiedScoringPlugin`

**Invariants:**
- All score-mutating systems live in exactly one plugin.
- No duplicate `ScoreSystem` writers.
- Salt miner logic co-located with score logic.
- Existing `ScoreSystem` API unchanged (no gameplay code change).

**Verification:**
- [ ] `cargo test` passes
- [ ] `grep -rn "ResMut<ScoreSystem>" src/` shows exactly one plugin owning all writes.
- [ ] Manual: score, combos, salt miner behave identically.

**Rollback:** Re-register both plugins.

---

### PR #8 — `arch/m02-plugins`: Plugin self-registration of resources

**Modified:**
- `src/main.rs` — remove manual `init_resource::<...>()` calls for resources owned by plugins
- Each of the six plugins' `build()` — add `init_resource` for their owned resources

**Invariants:**
- `main.rs` only registers plugins and top-level app configuration.
- Each plugin owns its resource initialization.
- No gameplay behavior change.

**Verification:**
- [ ] `cargo test` passes
- [ ] `cargo build` passes
- [ ] Manual: game launches and all resources initialize correctly.

**Rollback:** Move `init_resource` calls back to `main.rs`.

---

### PR #9 — `arch/m02-headless`: Headless app constructor

**New files:**
```
src/app_builder.rs              // headless_app() constructor, feature-gated
tests/headless_smoke.rs         // launch headless app, assert no panic
```

**Modified:**
- `Cargo.toml` — add optional `headless` feature
- `src/main.rs` — use `headless_app()` when feature is active

**Invariants:**
- Headless app has no window, no renderer, no audio — but all simulation/gameplay systems run.
- Headless app still processes `FixedUpdate` ticks.
- Existing native build unchanged.

**Verification:**
- [ ] `cargo test` passes
- [ ] `cargo test --features headless` passes
- [ ] CI runs headless smoke test.

**Rollback:** Remove `headless` feature and `app_builder.rs`.

---

### PR #10 — `arch/m02-simid`: Stable `SimId` component

**New files:**
```
src/simulation/sim_id.rs       // SimId(u64), SimIdGenerator resource
```

**Modified:**
- `src/simulation/mod.rs` — register `SimIdGenerator`, add `assign_sim_id_on_spawn` system
- `src/entities/` — add `SimId` to relevant spawn bundles (player, enemy, projectile, collectible)

**Invariants:**
- `SimId` is assigned at spawn and never changes.
- `SimId` is used for replay serialization, NOT for `Entity` references (Bevy's `Entity` is still the runtime handle).
- `SimId` is deterministic: same spawn order → same IDs when seeded.

**Verification:**
- [ ] `cargo test` passes
- [ ] New test: spawn 10 enemies, assert SimIds are 0..9.
- [ ] New test: same seed + same input → same SimId assignments.

**Rollback:** Remove `SimId` component and system.

---

### PR #11 — `arch/m02-replay`: Replay recording

**New files:**
```
src/replay/
  mod.rs              // ReplayPlugin
  recorder.rs         // Record inputs + events
  serializer.rs       // Save/load replay file
  playback.rs         // Replay playback controller
tests/replay.rs       // Round-trip test
```

**Modified:**
- `src/main.rs` — conditionally register `ReplayPlugin`
- `src/platform/` — input events forwarded to recorder
- `src/core/events.rs` — mark events as replay-serializable where needed

**Invariants:**
- Recording adds no per-frame allocation (pre-sized ring buffer or Vec).
- Replay file format is versioned.
- Playback produces identical `SimId` event sequences as live play (given same seed).
- Recording is off by default; toggled via command-line flag or menu.

**Verification:**
- [ ] `cargo test` passes
- [ ] New test: record 60 frames of input, playback, assert identical final score.
- [ ] Manual: record a session, playback shows identical combat.

**Rollback:** Remove `ReplayPlugin`.

---

### PR #12 — `arch/m02-state-hash`: Deterministic state hashing

**New files:**
```
src/replay/state_hash.rs        // Hash authoritative simulation state
```

**Modified:**
- `src/replay/mod.rs` — add state hash snapshot system
- `src/simulation/mod.rs` — run state hash after each `FixedUpdate` tick

**Invariants:**
- Hash includes: player stats, enemy stats (sorted by SimId), score, salt miner, spatial grid contents.
- Hash does NOT include: presentation state, audio state, camera transform.
- Same seed + same input → identical hash sequence.
- Hash is cheap enough to run every tick in release builds.

**Verification:**
- [ ] `cargo test` passes
- [ ] New test: run same scenario twice, assert identical hash at every tick.
- [ ] New test: replay playback produces identical hash sequence as recording.

**Rollback:** Remove state hash system.

---

## 4. PR Template (Use for Every PR)

```markdown
## Summary
What changed and why.

## Architecture impact
- Which plugin(s) gained/lost systems?
- Any new modules created?
- Any new events or resources introduced?

## Tests run
- `cargo test` result:
- `cargo build` result:
- Manual smoke test notes:
- Determinism test notes (if applicable):

## Known limitations
- Any systems temporarily duplicated?
- Any `TODO(Mission 3)` comments added?
- Any behavior you're not 100% sure is preserved?

## Rollback instructions
```bash
git revert <this-pr-commit> --no-edit
# or
git checkout main -- src/
```

## Specification changes
- None / Updated `MISSION_2_CANDIDATES.md` to reflect status
```

---

## 5. Dependency Resolution Rule

If the mission prompt and a normative document conflict:

1. **Stop.** Do not assume.
2. Implement the stricter of the two.
3. Flag the conflict in the PR description.
4. Propose an update to the conflicting document in the same PR.

---

## 6. Post-Mission 2 Success Criteria

Before declaring Mission 2 complete, verify:

- [ ] All simulation systems run in `FixedUpdate`.
- [ ] All gameplay timer/cooldown systems run in `FixedUpdate`.
- [ ] Presentation systems remain in `Update` (visual only).
- [ ] `SimulationRng` is seeded and deterministic; same seed → same outcome.
- [ ] `PresentationRng` is separate and unseeded.
- [ ] `ContactDetected` no longer carries detection-level data (split complete).
- [ ] Exactly one plugin owns `ScoreSystem` mutation.
- [ ] Each plugin self-registers its resources.
- [ ] Headless app constructor exists and runs in CI.
- [ ] `SimId` is assigned deterministically at spawn.
- [ ] Replay recording captures inputs + events and round-trips correctly.
- [ ] State hashing produces identical sequences for identical seeds.
- [ ] `cargo test --workspace` passes.
- [ ] `cargo build --workspace` passes.
- [ ] `cargo test --workspace --features headless` passes.
- [ ] Manual playthrough shows no observable gameplay change.
- [ ] All PRs merged to `main` with clean history.

---

*This document is append-only during Mission 2.*
