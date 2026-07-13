# Mission 1 — Architecture Stabilization: PR Checklist & Branch Strategy

**Mission:** Restructure existing codebase into six plugin boundaries without changing gameplay behavior.  
**Baseline commit:** `121c431ecc623ddc153a134e56aff3c185715321`  
**Branch prefix:** `arch/m01-*`  
**Governing docs:** `SIMULATION_CONTRACT.md`, `DEPENDENCY_RULES.md`, `CONTENT_SCHEMA.md`, `ENTITY_LIFECYCLE.md`, `ERROR_AND_DIAGNOSTICS.md`, `REPLAY_AND_REPRODUCTION.md`

---

## 1. Branch Strategy

```
main (protected)
  └── arch/m01-audit          ← PR #1: inventory only
        └── arch/m01-shells     ← PR #2: six empty plugins
              └── arch/m01-platform     ← PR #3
                    └── arch/m01-content      ← PR #4
                          └── arch/m01-diagnostics  ← PR #5
                                └── arch/m01-presentation ← PR #6
                                      └── arch/m01-gameplay     ← PR #7
                                            └── arch/m01-simulation   ← PR #8
                                                  └── arch/m01-collision-split ← PR #9 (incremental)
```

**Rules:**
- Each branch is a child of the previous merged branch.
- No force-push to shared branches.
- Every PR must compile and pass the Milestone 1 acceptance gate before review.

---

## 2. Acceptance Gate (Every PR)

```bash
# Must pass before opening PR
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo build --workspace

# Optional but recommended for PRs touching asset loading or WASM paths:
# cargo build --target wasm32-unknown-unknown --profile wasm-release
```

**Note on Clippy:** The baseline has global `#![allow(clippy::type_complexity)]` and `#![allow(clippy::too_many_arguments)]` in `main.rs` and `lib.rs`. For Mission 1, **do not remove these suppressions** — doing so would create a massive diff unrelated to plugin boundaries. Remove them in a dedicated cleanup milestone (Milestone 45 per `REBELLION_IMPLEMENTATION_PLAN.md`).

---

## 3. PR-by-PR Breakdown

### PR #1 — `arch/m01-audit`: Architecture audit current state

**Files:**
- `docs/migration/ARCHITECTURE_AUDIT_CURRENT_STATE.md` (new)
- `docs/migration/CURRENT_STATE.md` (new, short plugin/module map)

**Invariants:**
- Zero code changes.
- Zero `Cargo.toml` changes.

**Verification:**
- [ ] `cargo test` still passes (277 tests)
- [ ] `cargo build` still passes

**Rollback:** Revert documentation commit only.

---

### PR #2 — `arch/m01-shells`: Six empty plugin shells

**New files:**
```
src/simulation/
  mod.rs          // SimulationPlugin (empty impl)
src/gameplay/
  mod.rs          // GameplayPlugin (empty impl)
src/presentation/
  mod.rs          // PresentationPlugin (empty impl)
src/content/
  mod.rs          // ContentPlugin (empty impl)
src/platform/
  mod.rs          // PlatformPlugin (empty impl)
src/diagnostics/
  mod.rs          // DiagnosticsPlugin (empty impl)
```

**Modified:**
- `src/lib.rs` — declare new modules
- `src/main.rs` — register six new plugins alongside existing plugins

**Invariants:**
- Each plugin is a zero-system `Plugin` impl with a `build` that does nothing (or only inserts a placeholder resource).
- All existing plugins remain registered exactly as before.
- App builds and runs identically.

**Verification:**
- [ ] `cargo build` passes
- [ ] Manual smoke test: game launches, menu reachable, gameplay starts

**Rollback:** Remove module declarations and plugin registrations.

---

### PR #3 — `arch/m01-platform`: Move platform/input systems

**Source:** `SystemsPlugin` → `PlatformPlugin`

**Systems to move:**
- `JoystickPlugin` (gamepad input)
- `TouchJoystickPlugin` (mobile input)
- `pause_trigger_system` (ESC / Start button detection)

**Resources to move:**
- `InputConfig`
- `JoystickState`

**Invariants:**
- Input polling still works on native and WASM.
- Pause trigger still functions in `Playing` and `BossFight` states.
- No system left in `SystemsPlugin` that reads hardware input.

**Verification:**
- [ ] `cargo test` passes
- [ ] Manual: keyboard pause works
- [ ] Manual: gamepad input works (if available)

**Rollback:** Re-register moved plugins in `SystemsPlugin`; delete `PlatformPlugin` content.

---

### PR #4 — `arch/m01-content`: Move asset loading

**Source:** `AssetsPlugin` → `ContentPlugin`

**Systems to move:**
- `AssetsPlugin` (ship sprites, faction icons, powerup icons, etc.)
- Campaign metadata registration (currently in `GameModulesPlugin`, but *data-only* — keep executable campaign systems in `GameplayPlugin` for now)

**Resources to move:**
- `PowerupIconCache`
- Ship sprite caches
- Faction icon caches

**Invariants:**
- Asset loading order unchanged (still at app startup).
- No gameplay system directly imports content loader types.
- Campaign *selection data* (names, descriptions, factions) lives here; campaign *gameplay systems* stay in `GameplayPlugin`.

**Verification:**
- [ ] `cargo test` passes
- [ ] Manual: all sprites load correctly
- [ ] Manual: campaign select screen shows correct metadata

**Rollback:** Re-register `AssetsPlugin` in original location.

---

### PR #5 — `arch/m01-diagnostics`: Move debug/profiling

**Source:** `SystemsPlugin` → `DiagnosticsPlugin`

**Systems to move:**
- `PerfProfilePlugin` (frame time overlay)
- Any future diagnostic systems (entity counters, etc.)

**Resources to move:**
- (none currently; add entity-count resources here if they exist)

**Invariants:**
- Diagnostics are pure readers — no mutation of gameplay state.
- Overlay toggles correctly.

**Verification:**
- [ ] `cargo test` passes
- [ ] Manual: perf overlay visible and accurate

**Rollback:** Re-register `PerfProfilePlugin` in `SystemsPlugin`.

---

### PR #6 — `arch/m01-presentation`: Move visual/audio/FX/camera/UI

**Source:** `UiPlugin`, `SystemsPlugin` → `PresentationPlugin`

**Systems to move:**
- `HudPlugin`
- `MenuPlugin`
- `CapacitorWheelPlugin`
- `BackgroundPlugin`
- `TransitionPlugin`
- `EffectsPlugin` (screen shake, flashes, particles, explosions, starfield, damage numbers, hit flash)
- `AudioPlugin`
- `MusicPlugin`
- `DialoguePlugin`

**Resources to move:**
- `ScreenShake`
- `ScreenFlash`
- `CameraZoom`
- `HitStop`
- `AudioSettings`

**Invariants:**
- No presentation system mutates `ScoreSystem`, `EnemyStats`, `ShipStats`, or `SpatialGrid`.
- Effects still trigger from the same events/callbacks (we're only moving *where* they're registered, not *when* they run).
- `collision.rs` still spawns FX and sends events — that untangling happens in PR #9.

**Verification:**
- [ ] `cargo test` passes
- [ ] Manual: explosions, screen shake, music, dialogue all function identically
- [ ] Manual: menus and HUD render correctly

**Rollback:** Re-register all moved plugins in original locations.

---

### PR #7 — `arch/m01-gameplay`: Move AI, weapons, scoring, spawning, campaigns

**Source:** `EntitiesPlugin`, `SystemsPlugin` → `GameplayPlugin`

**Systems to move:**
- `PlayerPlugin` (movement, weapon firing — but keep projectile physics in `SimulationPlugin` boundary)
- `EnemyPlugin` (AI, pathing — but keep transform integration in `SimulationPlugin`)
- `WingmanPlugin`
- `DronePlugin`
- `AbilityPlugin`
- `SpawningPlugin`
- `BossPlugin`
- `ManeuverPlugin`
- `CampaignPlugin`
- `ScoringPlugin` (both of them — keep the duplication, just relocate)
- `SaltMinerSystem` resource registration

**Resources to move:**
- `ScoreSystem`
- `SaltMinerSystem`
- `GameProgress`
- `Difficulty`
- `CurrentStage`
- `ShipUnlocks`
- `CampaignState`
- `GameSession`
- `EndlessMode`

**Invariants:**
- Duplicate scoring authorities (`ScoringPlugin` + `ScoringSystemPlugin`) are **not resolved** — just co-located in `GameplayPlugin`.
- Campaign executable plugins (`ElderFleetPlugin`, etc.) are **not converted to data** — just registered through `GameplayPlugin` instead of `GameModulesPlugin`.
- No gameplay system mutates presentation resources directly.

**Verification:**
- [ ] `cargo test` passes
- [ ] Manual: full playthrough — score, combos, salt miner, abilities, bosses all behave identically
- [ ] Manual: campaign selection and progression unchanged

**Rollback:** Re-register all moved plugins in original locations.

---

### PR #8 — `arch/m01-simulation`: Move physics, collision detection, damage math

**Source:** `SystemsPlugin` → `SimulationPlugin`

**Systems to move:**
- `CollisionPlugin` — BUT: only the *detection* and *damage math* parts. The FX/scoring/dialogue parts stay where they are until PR #9.
- `ProjectilePlugin` (projectile movement/integration)
- `CollectiblePlugin` (if it handles physics; if purely presentation, send to `PresentationPlugin`)

**Resources to move:**
- `SpatialGrid`

**Invariants:**
- `SimulationPlugin` does **not** import `bevy_sprite`, `bevy_audio`, UI, or presentation modules.
- If `collision.rs` currently spawns sprites or plays sounds, those lines stay in the old file (or are duplicated with no-op stubs in `SimulationPlugin`) until PR #9.
- No system in `SimulationPlugin` writes to `ScoreSystem` or `SaltMinerSystem`.

**Verification:**
- [ ] `cargo test` passes
- [ ] Manual: collision detection still works; enemies die, projectiles despawn
- [ ] `grep -r "bevy_sprite\|bevy_audio\|bevy_ui" src/simulation/` returns nothing

**Rollback:** Re-register `CollisionPlugin` and `ProjectilePlugin` in `SystemsPlugin`.

---

### PR #9 — `arch/m01-collision-split`: Untangle `collision.rs` by responsibility

**This is the highest-risk PR. Split it into smaller sub-PRs if possible.**

**Strategy:** Instead of one big-bang rewrite, split `collision.rs` incrementally:

#### Sub-PR 9a: Extract detection-only system
- Create `detect_collisions.rs` in `src/simulation/` containing only `update_spatial_grid` + the contact-detection loop.
- The detection system emits a new event type (e.g., `ContactDetected`) instead of mutating health.
- Register in `SimulationPlugin`.

#### Sub-PR 9b: Extract damage resolution
- Create `resolve_damage.rs` in `src/simulation/` that consumes `ContactDetected` events and mutates `EnemyStats`/`ShipStats`.
- Does NOT spawn FX, send dialogue, or mutate score.
- Register in `SimulationPlugin`.

#### Sub-PR 9c: Extract death / despawn
- Create `resolve_deaths.rs` in `src/simulation/` that checks `health <= 0` and emits `EnemyDestroyedEvent` (already exists).
- Does NOT mutate score or spawn drops.
- Register in `SimulationPlugin`.

#### Sub-PR 9d: Extract presentation reactions
- Move FX spawning, screen shake, screen flash, camera zoom, hitstop, dialogue trigger, floating damage numbers from `collision.rs` into `PresentationPlugin` systems that consume `ContactDetected` / `EnemyDestroyedEvent` / `PlayerDamagedEvent`.

#### Sub-PR 9e: Extract score / drops / game-over
- Move score mutation, salt miner updates, powerup drops, liberation pods, and `GameState::GameOver` transition from `collision.rs` into `GameplayPlugin` systems that consume `EnemyDestroyedEvent` / `PlayerDamagedEvent`.

**Invariants:**
- The game plays **identically** before and after each sub-PR.
- After 9e, `collision.rs` is deleted or reduced to a thin `SpatialGrid` utility.
- No system in `SimulationPlugin` spawns sprites, plays audio, mutates score, or triggers dialogue.
- No system in `PresentationPlugin` mutates health or score.
- No system in `GameplayPlugin` performs collision detection or spatial grid queries.

**Verification (per sub-PR):**
- [ ] `cargo test` passes
- [ ] Manual playthrough: combat feels identical
- [ ] `grep -rn "spawn_impact_sparks\|spawn_damage_number\|screen_shake\|screen_flash\|camera_zoom\|hit_stop\|dialogue_events" src/simulation/` returns nothing
- [ ] `grep -rn "enemy_stats.health -=\|player_stats.take_damage" src/gameplay/` returns nothing (damage math lives in `simulation`)

**Rollback:** Any sub-PR can be reverted independently. Keep `collision.rs` as a `legacy_collision.rs` module during the transition if needed for comparison.

---

## 4. PR Template (Use for Every PR)

```markdown
## Summary
What moved and why.

## Architecture impact
- Which plugin(s) gained/lost systems?
- Any new modules created?
- Any new events or resources introduced?

## Tests run
- `cargo test` result:
- `cargo build` result:
- Manual smoke test notes:

## Known limitations
- Any systems temporarily duplicated?
- Any `TODO(Mission 2)` comments added?
- Any behavior you're not 100% sure is preserved?

## Rollback instructions
```bash
git revert <this-pr-commit> --no-edit
# or
git checkout main -- src/
```

## Specification changes
- None / Updated `ARCHITECTURE_AUDIT_CURRENT_STATE.md` to reflect new layout
```

---

## 5. Conflict Resolution Rule

If the mission prompt (`CLAUDE_CODE_MISSION_1.md`) and a normative document (`SIMULATION_CONTRACT.md`, etc.) conflict:

1. **Stop.** Do not assume.
2. Implement the stricter of the two.
3. Flag the conflict in the PR description.
4. Propose an update to the conflicting document in the same PR.

---

## 6. Post-Mission 1 Success Criteria

Before declaring Mission 1 complete, verify:

- [ ] `ARCHITECTURE_AUDIT_CURRENT_STATE.md` accurately reflects the new plugin layout.
- [ ] `MISSION_2_CANDIDATES.md` exists and lists every determinism/timestep/event-pattern issue noticed but not fixed.
- [ ] Dependency direction matches `DEPENDENCY_RULES.md` §1 as closely as single-crate structure allows.
- [ ] No circular dependencies between the six plugins.
- [ ] Every system lives in exactly one plugin.
- [ ] No duplicate gameplay logic introduced (existing duplicates were moved, not resolved).
- [ ] `cargo test --workspace` passes.
- [ ] `cargo build --workspace` passes.
- [ ] Manual playthrough shows no observable gameplay change.
- [ ] All PRs merged to `main` with clean history.

---

*This document is append-only during Mission 1.*
