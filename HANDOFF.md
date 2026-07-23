# Handoff: Triglavian Campaign + Abyssal Depths Build Sprint

**Branch**: `feat/triglavian-campaign-20260723`  
**Date**: 2026-07-23  
**Status**: Active — all tests passing (303 unit + 15 integration), clippy clean  
**Last commit**: `dbda8f2` — Abyssal Depths room progression integration tests

---

## What Was Built

### 1. Triglavian Invasion Campaign (`src/games/triglavian_invasion/`)

- **`games/triglavian_invasion/config/module.json`** (NEW)
  - Complete 9-mission module config with ship pools (EDENCOM + Triglavian),
    enemy spawn weights, mission definitions, boss stat blocks, epilogue text.
- **`campaign.rs`** — Boss mechanics fully implemented:
  - `trig_boss_intro()`: Boss descends to y=200, transitions to BossFight after 2s.
  - `update_trig_boss()`: Sweep movement, 3-phase transitions, enrage at 20% health.
  - Attack patterns: 360° ring (XORDAZH), spread shot (LESHAK/IKITURSA), aimed single.
  - `spawn_trig_projectile()` helper with `EnemyProjectile` + `ProjectileDamage`.
  - `spawn_trig_boss()` now spawns with `Enemy` + `EnemyStats` for collision compatibility.
- **`mod.rs`** — Victory screen wired:
  - `spawn_trig_victory_screen` with faction-specific content (EDENCOM blue / Triglavian red).
  - Input handling (Space/ESC → MainMenu).
  - Boss intro UI with pulsing warning text, boss name card, phase indicator.
- **`ships.rs`** — 5 unit tests for stats, spawn weights, progression.
- **`mod.rs`** — High score persistence wired into victory screen (matches CG pattern).
- **`tests/triglavian_boss_phases.rs`** (NEW) — Integration tests:
  - `trig_boss_phase_transition_and_enrage`: Phase 1→2→3 at correct thresholds,
    speed increases (×1.2 per phase), enrage at 20% (×1.5).
  - `trig_boss_spawns_projectiles_during_boss_fight`: Leshak fires 5-bullet spread.
  - `trig_boss_does_not_fire_when_not_in_battle_state`: Intro state suppresses firing.

### 2. Abyssal Depths Campaign (`src/games/abyssal_depths/`)

- **`games/abyssal_depths/config/module.json`** (NEW)
  - 3-room survival mode config: POCKET (8 enemies) → ESCALATION (12) → EXTRACTION (15 + Drekavac boss).
  - 11 cross-faction player ships, 4 enemy types, time limit, extraction channel time.
- **`mod.rs`** — Fixes and features:
  - `.chain()` added to Update systems to prevent race conditions between
    `check_room_clear` and `handle_extraction`.
  - 8 unit tests for `AbyssalState` and `AbyssalRoom`.
  - Victory screen wired: extraction success with loot display.
- High score persistence wired into victory screen (matches CG pattern).
- **`tests/abyssal_room_progression.rs`** (NEW) — Integration tests:
  - `abyssal_room1_clears_and_spawns_transition_gate`: Room clear detection + gate spawn.
  - `abyssal_room3_extraction_gate_triggers_victory`: Extraction channeling accumulates
    progress and marks `extracted = true`.
  - `abyssal_timer_runs_out_triggers_game_over`: Timer expiry transitions to `GameOver`.

### 3. Core Engine Fix — BossFight State Damage Gap

**Critical bug fixed**: Simulation collision/damage systems and ALL core gameplay
systems (player, enemy, projectile, effects, audio, music, HUD, abilities,
maneuvers, scoring, collectibles) were gated to `GameState::Playing` only.
Campaigns using `BossIntro`/`BossFight` (Triglavian, Caldari-Gallente) had
unplayable boss fights — no input, no damage, no visuals.

- `src/simulation/mod.rs` — `simulation_active` condition matches `Playing | BossFight`.
- 17 files updated with `.or(in_state(GameState::BossFight))`.
- Wave spawning (`systems/spawning.rs`) correctly remains `Playing`-only.
- Elder Fleet unaffected (never enters `BossFight`).
- **`tests/boss_fight_e2e.rs`** (NEW) — Regression integration tests:
  - `projectile_deals_damage_during_boss_fight`: Validates collision/damage
    systems run during BossFight by spawning enemy, firing projectile, asserting
    health reduction and state hash change.
  - `player_can_move_during_boss_fight`: Spawns player with Movement component,
    verifies 10 ticks pass without panic during BossFight.

---

## Known Issues / Next Steps

### P0 — Must Fix Before Merge
- [x] **Integration test for BossFight state** — `tests/boss_fight_e2e.rs` added.
- [x] **Integration test for Triglavian boss phases** — `tests/triglavian_boss_phases.rs` added.
- [x] **Integration test for Abyssal Depths room progression** — `tests/abyssal_room_progression.rs` added.

### P1 — Polish
- [x] **Triglavian boss intro UI** — Boss intro screen with warning text, boss name,
  mission name, phase indicator, "Prepare for battle..." (commit `1897c32`).
- [ ] **Caldari-Gallente BossFight validation** — CG also uses `BossIntro`/`BossFight`.
  The core engine fix helps, but verify CG boss is actually playable now.
- [x] **High score persistence** — Triglavian and Abyssal victory screens now
  persist scores to `SaveData` (matches CG pattern).

### P2 — Content Expansion
- [ ] **Elder Fleet module.json** — Only campaign without a JSON config module.
  Could add `games/elder_fleet/config/module.json` for consistency.
- [ ] **Additional enemy variants** — `EnemyVariant` supports `BlindingVedmak`,
  `StarvingDamavik`, etc. Could add more variety to Triglavian missions.
- [ ] **Mission-specific objectives** — Currently missions only track "survive waves."
  Could add escort, timed survival, or kill-count objectives.

---

## How to Resume

```bash
cd /home/arete/projects/rebellion
git checkout feat/triglavian-campaign-20260723
cargo test && cargo clippy -- -D warnings
```

---

## Session Notes

- BossFight state bug was the biggest blocker. Fixing it required touching 17
  core files but was essential for any boss-fight campaign to be playable.
- All `EnemyVariant` references used in Triglavian and Abyssal spawns verified
  against `src/entities/enemy/spawn.rs` — all valid.
- Bevy 0.15 `.or()` condition chaining confirmed working (used in
  `platform/mod.rs` already).
- Default path for Abyssal Depths (`AbyssalState::default()`) uses Room1 with
  600s timer — confirmed in tests.
- Headless test note: `ButtonInput::just_pressed` never auto-clears without
  Bevy's input plugin. Use `clear_just_pressed` manually or avoid asserting
  state transitions that depend on `just_pressed` in headless mode.
