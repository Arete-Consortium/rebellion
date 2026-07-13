# Architecture Audit — Current State

**Repository:** `Arete-Consortium/rebellion`
**Baseline Commit:** `121c431ecc623ddc153a134e56aff3c185715321`
**Mission 1 Complete Commit:** `bc60c79`
**Date:** 2026-07-12
**Auditor:** Claude Code (Mission 1)
**Status:** Mission 1 complete — six plugin boundaries established, `collision.rs` untangled  

---

## 1. Build & Test Baseline

| Target | Status | Notes |
|--------|--------|-------|
| Native (`cargo build`) | ✅ Pass | 2 warnings: unused imports `EffectiveStats`, `Inventory` in `src/entities/mod.rs` |
| Tests (`cargo test`) | ✅ Pass | 277 passed, 0 failed |
| WASM (`cargo build --target wasm32-unknown-unknown`) | ⚠️ TBD | Not run in this session; `build-wasm.sh` exists |
| Format (`cargo fmt --check`) | ⚠️ TBD | Not run yet |
| Clippy (`cargo clippy -- -D warnings`) | ⚠️ TBD | Global `allow(clippy::type_complexity)` and `allow(clippy::too_many_arguments)` in `main.rs` and `lib.rs` will fail narrow gate |

**Known defects from prior audit (unconfirmed live):**
- `collision.rs` mixes detection, damage, death, scoring, FX, dialogue, camera, hitstop
- Duplicate scoring: `ScoringPlugin` + `ScoringSystemPlugin` both registered
- No `FixedUpdate` usage; all gameplay runs in frame-based `Update`
- Campaigns are executable plugins (`ElderFleetPlugin`, `CaldariGallentePlugin`, etc.) not data
- `fastrand` global RNG used in collision and effects

---

## 2. Plugin Inventory (Post-Mission 1)

```text
main.rs
├── DefaultPlugins + WindowPlugin + EguiPlugin
├── GameState (bevy_state)
├── Resources (11+ global)
├── Events (5 campaign events)
└── Plugin group
    ├── SavePlugin
    ├── AnalyticsPlugin
    ├── AchievementPlugin
    ├── ContentPlugin          ← was AssetsPlugin
    │   └── (asset loading, caches)
    ├── GameEventsPlugin
    ├── SimulationPlugin       ← NEW
    │   ├── SpatialGrid
    │   ├── detect_collisions.rs   (ContactDetected events)
    │   ├── resolve_damage.rs      (health math, burn, chain, pierce)
    │   ├── resolve_deaths.rs        (health ≤ 0, EnemyDestroyedEvent)
    │   ├── ProjectilePlugin
    │   └── CollectiblePlugin
    ├── GameplayPlugin         ← NEW
    │   ├── PlayerPlugin
    │   ├── EnemyPlugin
    │   ├── WingmanPlugin
    │   ├── DronePlugin
    │   ├── AbilityPlugin
    │   ├── SpawningPlugin
    │   ├── BossPlugin
    │   ├── ManeuverPlugin
    │   ├── CampaignPlugin
    │   ├── ScoringPlugin          ← DUPLICATE AUTHORITY #1 (co-located)
    │   ├── ScoringSystemPlugin      ← DUPLICATE AUTHORITY #2 (co-located)
    │   └── combat_outcomes.rs     (score, salt miner, drops, game over)
    ├── PresentationPlugin     ← NEW
    │   ├── EffectsPlugin
    │   ├── AudioPlugin
    │   ├── MusicPlugin
    │   ├── DialoguePlugin
    │   ├── UiPlugin
    │   │   ├── HudPlugin
    │   │   ├── MenuPlugin
    │   │   ├── CapacitorWheelPlugin
    │   │   ├── BackgroundPlugin
    │   │   └── TransitionPlugin
    │   └── combat_reactions.rs    (FX, screen shake, hit flash, damage numbers)
    ├── PlatformPlugin         ← NEW
    │   ├── JoystickPlugin
    │   └── TouchJoystickPlugin
    ├── DiagnosticsPlugin      ← NEW
    │   └── PerfProfilePlugin
    └── GameModulesPlugin
        ├── ElderFleetPlugin
        ├── CaldariGallentePlugin
        ├── AbyssalDepthsPlugin
        └── TriglavianInvasionPlugin
```

---

## 3. Resource / Component Ownership Map

| Resource | Type | Writers | Readers | Target Plugin (Mission 1) |
|----------|------|---------|---------|---------------------------|
| `ScoreSystem` | `Resource` | `ScoringPlugin`, `ScoringSystemPlugin`, `combat_outcomes.rs` | HUD, results screen | `GameplayPlugin` |
| `SaltMinerSystem` | `Resource` | `combat_outcomes.rs` | HUD | `GameplayPlugin` |
| `GameProgress` | `Resource` | Save, campaign, scoring | UI | `GameplayPlugin` |
| `InputConfig` | `Resource` | Settings menu | Input systems | `PlatformPlugin` |
| `AudioSettings` | `Resource` | Settings menu | Audio systems | `PresentationPlugin` |
| `Difficulty` | `Resource` | Menu | Spawning, damage | `GameplayPlugin` |
| `SelectedShip` | `Resource` | Ship select UI | Player spawn | `ContentPlugin` → `GameplayPlugin` |
| `CurrentStage` | `Resource` | Campaign, spawning | UI | `GameplayPlugin` |
| `ShipUnlocks` | `Resource` | Campaign, achievements | UI | `GameplayPlugin` |
| `CampaignState` | `Resource` | Campaign systems | UI | `GameplayPlugin` |
| `GameSession` | `Resource` | Campaign, game over | UI, analytics | `GameplayPlugin` |
| `EndlessMode` | `Resource` | Menu | Spawning | `GameplayPlugin` |
| `SpatialGrid` | `Resource` | `detect_collisions.rs` (clear+insert) | `resolve_damage.rs` (query) | `SimulationPlugin` |
| `ScreenShake` | `Resource` | `combat_reactions.rs` | Camera/effects | `PresentationPlugin` |
| `ScreenFlash` | `Resource` | `combat_reactions.rs` | Camera/effects | `PresentationPlugin` |
| `CameraZoom` | `Resource` | `combat_reactions.rs` | Camera | `PresentationPlugin` |
| `HitStop` | `Resource` | `combat_reactions.rs` | Time/effects | `PresentationPlugin` |
| `JoystickState` | `Resource` | `JoystickPlugin` | Pause system, player | `PlatformPlugin` |

**Components (selected — full inventory deferred to per-system pass):**
- `Player`, `Enemy`, `Projectile`, `Collectible`, `Boss`, `Wingman`, `Drone`
- `Transform`, `Velocity` — written by movement and collision systems
- `EnemyStats`, `ShipStats` — written by collision (damage), spawning
- `PowerupEffects`, `ManeuverState` — read by collision for invulnerability

---

## 4. System Registration Map

All systems currently run in `Update` unless noted.

### 4.1 `EntitiesPlugin` systems

| System | Registers In | Description | Target Plugin |
|--------|-------------|-------------|---------------|
| (player movement/input) | `PlayerPlugin` | Player ship control | `GameplayPlugin` / `PlatformPlugin` |
| (enemy AI/movement) | `EnemyPlugin` | Enemy behavior | `GameplayPlugin` |
| (projectile spawn/move) | `ProjectilePlugin` | Projectiles | `SimulationPlugin` |
| (collectible spawn) | `CollectiblePlugin` | Power-ups | `GameplayPlugin` |
| (wingman AI) | `WingmanPlugin` | Wingman behavior | `GameplayPlugin` |
| (drone AI) | `DronePlugin` | Drone behavior | `GameplayPlugin` |

### 4.2 `SimulationPlugin` systems

| System | Registers In | Description | Status |
|--------|-------------|-------------|--------|
| `update_spatial_grid` | `SimulationPlugin` | Spatial partitioning | ✅ Moved |
| `detect_player_projectile_hits` | `SimulationPlugin` | Detection loop (emits `ContactDetected`) | ✅ New (PR #9a) |
| `detect_enemy_projectile_hits` | `SimulationPlugin` | Detection loop (emits `ContactDetected`) | ✅ New (PR #9a) |
| `resolve_player_projectile_damage` | `SimulationPlugin` | Damage math, burn, chain, pierce | ✅ New (PR #9b) |
| `resolve_enemy_projectile_damage` | `SimulationPlugin` | Player damage with layer breakdown | ✅ New (PR #9b) |
| `resolve_enemy_deaths` | `SimulationPlugin` | Health ≤ 0 check, `EnemyDestroyedEvent`, despawn | ✅ New (PR #9c) |

### 4.3 `GameplayPlugin` systems (collision outcomes)

| System | Registers In | Description | Status |
|--------|-------------|-------------|--------|
| `enemy_death_outcomes` | `GameplayPlugin` | Score, salt miner, drops, liberation pods | ✅ New (PR #9e) |
| `player_damage_outcomes` | `GameplayPlugin` | No-damage bonus lost | ✅ New (PR #9e) |
| `player_death_outcome` | `GameplayPlugin` | `GameState::GameOver` transition | ✅ New (PR #9e) |

### 4.4 `PresentationPlugin` systems (combat reactions)

| System | Registers In | Description | Status |
|--------|-------------|-------------|--------|
| `enemy_hit_reactions` | `PresentationPlugin` | Impact sparks, damage numbers, hit flash, screen shake | ✅ New (PR #9d) |
| `boss_health_callouts` | `PresentationPlugin` | Boss low-health dialogue | ✅ New (PR #9d) |
| `enemy_death_reactions` | `PresentationPlugin` | Explosions, screen effects, camera zoom | ✅ New (PR #9d) |
| `player_hit_reactions` | `PresentationPlugin` | Hit flash, rumble, screen shake, health callouts | ✅ New (PR #9d) |
| `player_death_reactions` | `PresentationPlugin` | Death FX (explosion, screen flash/shake) | ✅ New (PR #9d) |
| `spawn_chain_bolts` | `PresentationPlugin` | Chain lightning sprite spawn | ✅ Moved (PR #9d) |
| `tick_chain_bolts` | `PresentationPlugin` | Chain bolt FX lifecycle | ✅ Moved (PR #9d) |
| `pause_trigger_system` | `SystemsPlugin` | ESC / Start → Pause | `PlatformPlugin` |
| (scoring v1) | `ScoringPlugin` | Score/combo logic | `GameplayPlugin` |
| (scoring v2) | `ScoringSystemPlugin` | ComboHeatSystem, SaltMiner | `GameplayPlugin` |
| (effects spawners) | `EffectsPlugin` | Screen shake, flashes, particles | `PresentationPlugin` |
| (spawning) | `SpawningPlugin` | Wave/enemy spawn logic | `GameplayPlugin` |
| (boss logic) | `BossPlugin` | Boss phases, attacks | `GameplayPlugin` |
| (dialogue) | `DialoguePlugin` | Triggered dialogue lines | `PresentationPlugin` |
| (audio) | `AudioPlugin` | SFX playback | `PresentationPlugin` |
| (music) | `MusicPlugin` | Music transitions | `PresentationPlugin` |
| (joystick input) | `JoystickPlugin` | Gamepad input | `PlatformPlugin` |
| (touch joystick) | `TouchJoystickPlugin` | Mobile input | `PlatformPlugin` |
| (perf profile) | `PerfProfilePlugin` | Frame time overlay | `DiagnosticsPlugin` |
| (maneuvers) | `ManeuverPlugin` | Barrel rolls, dodges | `GameplayPlugin` |
| (campaign state) | `CampaignPlugin` | Mission progression | `GameplayPlugin` |
| (abilities) | `AbilityPlugin` | Special abilities | `GameplayPlugin` |

### 4.3 `UiPlugin` systems

| System | Registers In | Description | Target Plugin |
|--------|-------------|-------------|---------------|
| (hud render) | `HudPlugin` | Score, health, minimap | `PresentationPlugin` |
| (menus) | `MenuPlugin` | Main menu, pause, settings | `PresentationPlugin` |
| (capacitor wheel) | `CapacitorWheelPlugin` | Energy UI | `PresentationPlugin` |
| (backgrounds) | `BackgroundPlugin` | Parallax, starfields | `PresentationPlugin` |
| (transitions) | `TransitionPlugin` | Screen wipes | `PresentationPlugin` |

### 4.4 `GameModulesPlugin` systems

| System | Registers In | Description | Target Plugin |
|--------|-------------|-------------|---------------|
| (elder fleet campaign) | `ElderFleetPlugin` | Campaign-specific logic | `ContentPlugin` (data) / `GameplayPlugin` (systems) |
| (caldari-gallente) | `CaldariGallentePlugin` | Campaign-specific logic | `ContentPlugin` (data) / `GameplayPlugin` (systems) |
| (abyssal depths) | `AbyssalDepthsPlugin` | Campaign-specific logic | `ContentPlugin` (data) / `GameplayPlugin` (systems) |
| (triglavian) | `TriglavianInvasionPlugin` | Campaign-specific logic | `ContentPlugin` (data) / `GameplayPlugin` (systems) |

---

## 5. RNG / Randomness Usage

| Location | Source | Purpose | Target Fix (Mission 2+) |
|----------|--------|---------|---------------------------|
| `src/systems/collision.rs:133` | `fastrand::f32()` | Chain bolt zigzag jitter | `PresentationRng` (cosmetic) |
| `src/systems/collision.rs:257` | `fastrand::f32()` | Critical hit roll | `SimulationRng` |
| `src/systems/collision.rs:426` | `fastrand::f32()` | Powerup drop chance | `SimulationRng` |
| `src/systems/effects/*.rs` | (assumed) | Particle variance | `PresentationRng` |

**Finding:** `rand` crate is in `Cargo.toml` but `fastrand` is actively used in collision. No seeded `SimulationRng` exists.

---

## 6. Wall-Clock / Frame-Rate Coupling

| Location | Mechanism | Risk |
|----------|-----------|------|
| All systems in `Update` | Frame-based execution | Combat varies by FPS |
| `tick_chain_bolts` | `time.delta_secs()` | FX lifetime ok (cosmetic), but pattern established |
| `last_callout` cooldown | `time.delta_secs()` | Health callout timing varies by FPS |
| Cooldowns / timers | (assumed `Time` or manual) | Need per-system audit in Mission 2 |

**Finding:** No `FixedUpdate` usage found. No `Time<Fixed>` configuration.

---

## 7. `collision.rs` Cross-Domain Touch Points

The `player_projectile_enemy_collision` system directly touches:

1. **Detection** (`SpatialGrid` lookup) — belongs in `SimulationPlugin`
2. **Damage math** (crit roll, ammo mult, health subtraction) — belongs in `SimulationPlugin`
3. **Boss dialogue trigger** — belongs in `PresentationPlugin`
4. **Hit flash** (sprite mutation) — belongs in `PresentationPlugin`
5. **Impact sparks** (FX spawn) — belongs in `PresentationPlugin`
6. **Screen flash / shake / zoom / hitstop** — belongs in `PresentationPlugin`
7. **Floating damage numbers** — belongs in `PresentationPlugin`
8. **Burn DoT application** (component insert) — belongs in `SimulationPlugin`
9. **Chain lightning target planning + FX spawn** — split: planning in `SimulationPlugin`, bolts in `PresentationPlugin`
10. **Score mutation** (`score.on_kill`, `salt_miner.on_kill_at_distance`) — belongs in `GameplayPlugin`
11. **Enemy destruction event** — belongs in `SimulationPlugin` (emits fact)
12. **Explosion event** — belongs in `PresentationPlugin` (reads fact)
13. **Liberation pod spawn** — belongs in `GameplayPlugin`
14. **Powerup drop** — belongs in `GameplayPlugin`
15. **Enemy despawn** — belongs in `SimulationPlugin` (`Cleanup` phase)

The `enemy_projectile_player_collision` system directly touches:

1. **Detection** — `SimulationPlugin`
2. **Damage application** (`take_damage_detailed`) — `SimulationPlugin`
3. **Damage layer events** (for FX) — `PresentationPlugin`
4. **Hit flash** — `PresentationPlugin`
5. **Score no-damage bonus flag** — `GameplayPlugin`
6. **Rumble event** — `PlatformPlugin`
7. **Screen shake** — `PresentationPlugin`
8. **Health callout dialogue** — `PresentationPlugin`
9. **Death / GameOver state transition** — `GameplayPlugin`
10. **Explosion event** — `PresentationPlugin`

---

## 8. Global / Hidden State

| Item | Location | Severity | Action for Mission 1 |
|------|----------|----------|----------------------|
| Global Clippy suppressions | `main.rs`, `lib.rs` | P2 | Flag; remove in cleanup milestone |
| `fastrand` global seed | implicit | P1 | Flag for Mission 2 |
| `console_error_panic_hook` | `main.rs` WASM gate | OK | Keep in `PlatformPlugin` |

---

## 9. Dependency Direction Analysis (Post-Mission 1)

**Resolved violations:**

- `collision.rs` no longer imports `crate::assets::PowerupIconCache` — removed
- `collision.rs` no longer imports `super::effects::*` — removed
- `collision.rs` no longer imports `super::DialogueEvent` — removed
- `collision.rs` no longer writes to `ScoreSystem` or `SaltMinerSystem` — removed

**Remaining structural risks (deferred to Mission 2+):**

- `systems/mod.rs` glob-re-exports everything, making cross-domain imports trivial — **structural risk**
- `main.rs` initializes 11+ resources manually — needs plugin self-registration

---

## 10. Mission 2 Candidates

| ID | Issue | Location | Status |
|----|-------|----------|--------|
| ~~M2-001~~ | No `FixedUpdate` / `SimSet` | Entire codebase | **Deferred** — Mission 2 scope |
| ~~M2-002~~ | Unseeded `fastrand` in crit/drops | `resolve_damage.rs` | **Deferred** — Needs `SimulationRng` + `MissionSeed` |
| ~~M2-003~~ | Chain bolt jitter uses gameplay RNG | `combat_reactions.rs` | **Deferred** — Needs `PresentationRng` separation |
| ~~M2-004~~ | Frame-rate dependent cooldowns | Assumed across systems | **Deferred** — Needs `Time<Fixed>` migration |
| ~~M2-005~~ | `collision.rs` performs scoring | `collision.rs` | **RESOLVED** — Extracted to `combat_outcomes.rs` (PR #9e) |
| M2-006 | Duplicate score authority | `ScoringPlugin` + `ScoringSystemPlugin` | **Deferred** — Needs unified scoring design |
| ~~M2-007~~ | Campaign plugins executable | `games/` | **Deferred** — Mission 4+ scope |
| M2-008 | `main.rs` initializes 11+ resources manually | `main.rs` | **Deferred** — Needs plugin self-registration |
| ~~M2-009~~ | No headless app constructor | Missing | **Deferred** — Mission 2 scope |
| ~~M2-010~~ | No `SimId` stable IDs | Missing | **Deferred** — Mission 2 scope |
| ~~M2-011~~ | No replay recording | Missing | **Deferred** — Mission 3 scope |
| ~~M2-012~~ | No state hashing | Missing | **Deferred** — Mission 3 scope |

**New findings from PR #9:**

| ID | Issue | Location | Why Deferred |
|----|-------|----------|--------------|
| M2-013 | `ContactDetected` carries raw crit_chance/ammo_type | `core/events.rs` | Detection shouldn't know resolution details; could split into raw + resolved event |
| M2-014 | `BossCalloutSent` uses component marker instead of resource | `combat_reactions.rs` | Works for single-boss case; revisit if multi-boss fights added |

---

## 11. Rollback Point

- **Pre-Mission 1 commit:** `121c431ecc623ddc153a134e56aff3c185715321`
- **Tag:** `rebellion-2-baseline` (to be created)
- **Build status at baseline:** Native builds, 277 tests pass, 2 unused-import warnings
- **WASM status at baseline:** Unknown — run `bash build-wasm.sh` to verify

---

## 12. Lore/Content Correction — Caldari-Gallente Campaign Structure

**Correction to initial audit:** The `caldari_gallente` module is **not** misnamed. It is a multi-act campaign called "Battle of Caldari Prime" with the following structure:

| Act | Mission ID | Name | Description |
|-----|-----------|------|-------------|
| 1 | `cg_m1_orbital_skirmish` | ORBITAL SKIRMISH | First contact; tutorial |
| 2 | `cg_m2_urban_firefight` | URBAN FIREFIGHT | City combat above Caldari Prime |
| 3 | `cg_m3_fleet_interdiction` | FLEET INTERDICTION | Intercept reinforcements |
| 4 | `cg_m4_escalation` | ESCALATION POINT | T3 destroyers enter; T3 ship unlock |
| 5 | `cg_m5_decisive_push` | DECISIVE PUSH | Final battle for orbital superiority |
| Epilogue | `cg_epilogue_shiigeru` | FINAL DIRECTIVE: SHIIGERU | Endless nightmare survival mode aboard the dying titan |

The storyline doc `CAMPAIGN_3_THE_LAST_STAND.md` documents **only the Shiigeru epilogue** (Operation Highlander, YC115). The preceding 5 missions constitute the "Caldari-Gallente war chapter" that builds up to that finale.

**Implication for content migration (Milestones 30–33):** The RON conversion for this campaign must include all 5 main missions plus the Shiigeru nightmare configuration — not just the Shiigeru finale. The `campaign.rs` file already defines `CGMission` structs, `CG_MISSIONS` array, `NightmareBoss` enum, and `ShiigeruNightmare` state machine, which form a ready-made data skeleton.

---

## 13. Storyline-to-Code Campaign Mapping

| Storyline Doc | Campaign Plugin (Code) | Status |
|---------------|------------------------|--------|
| `CAMPAIGN_1_MINMATAR_REBELLION.md` | `elder_fleet::ElderFleetPlugin` | ✅ Aligned |
| `CAMPAIGN_2_SANSHA_INCURSIONS.md` | **None** — no `sansha_invasion` plugin in repo | ⚠️ **Orphan doc** |
| `CAMPAIGN_3_THE_LAST_STAND.md` | `caldari_gallente::CaldariGallentePlugin` (Shiigeru epilogue only) | ⚠️ Doc covers only final act; full 5-mission arc is in code |
| `CAMPAIGN_4_ABYSSAL_DEPTHS.md` | `abyssal_depths::AbyssalDepthsPlugin` | ✅ Aligned |
| — | `triglavian_invasion::TriglavianInvasionPlugin` | ⚠️ **Orphan plugin** — no matching storyline doc |

**Findings:**
- The **Sansha campaign** (`CAMPAIGN_2_SANSHA_INCURSIONS.md`) is an **aspirational/out-of-scope** storyline document — confirmed by project owner as "shelved until we build the rest of this." No executable plugin exists; no work required for v3.0.
- The **Triglavian Invasion** plugin exists in code but has no matching storyline document in the extracted package. The `CAMPAIGN_4_ABYSSAL_DEPTHS.md` covers the Triglavian emergence/Proving/Pochven arc, which may map to this plugin, but the numbering mismatch (Campaign 4 doc vs. `triglavian_invasion` plugin name) needs clarification.

**Implication for content migration:** When converting campaigns to RON data (Milestones 30–33):
- **Scope for v3.0:** Elder Fleet, Caldari-Gallente (incl. Shiigeru), Abyssal Depths/Triglavian
- **Out of scope:** Sansha campaign — no code, no data conversion needed
- Ensure every in-scope code plugin has a corresponding narrative source document before RON conversion begins

---

*This document is append-only during Mission 1. Update as systems move.*
