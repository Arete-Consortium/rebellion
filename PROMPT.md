# Senior Engineer Loop Prompt: rebellion v2.0.0 Build Sprint

**Branch**: `feat/triglavian-campaign-20260723`
**Status**: P0/P1/P2 HANDOFF items complete. All 624 tests green, clippy clean.
**Goal**: Maximum build output. Ship concrete, testable improvements.

## Current State

### Completed This Session
- ✅ CG BossFight validation: `tests/cg_boss_phases.rs` (3 tests)
- ✅ Elder Fleet module.json: full Minmatar vs Amarr campaign config
- ✅ Enemy variants: `Kikimora` + `Leshak` added to `EnemyVariant`, wired into Triglavian spawning
- ✅ Objective tracking fix: `no_damage_taken` tracks damage, `bonus_complete` evaluated at mission end, `BossType::None` missions transition correctly

### Test Summary
```
18 integration tests (boss_fight_e2e, cg_boss_phases, triglavian_boss_phases, abyssal_room_progression, etc.)
~606 unit tests across modules
 cargo test        → all green
 cargo clippy -- -D warnings  → clean
```

## Next Priority Stack (highest ROI first)

### 1. Integration Tests for New Fixes (protect the work)
Add headless integration tests in `tests/` for:
- `player_damage_sets_no_damage_taken_false` — spawn player, simulate projectile hit, assert `CampaignState.no_damage_taken == false` after `player_damage_outcomes`
- `no_boss_mission_completes_without_boss` — configure `CampaignState` with `BossType::None` mission, run `spawn_next_wave` through all waves, assert `NextState(StageComplete)`
- `bonus_complete_when_no_damage_taken` — complete mission without damage, assert `bonus_complete == true`

### 2. Elder Fleet Campaign Gameplay Systems
Elder Fleet has `mod.rs` + `ships.rs` but no actual mission gameplay. Add:
- `ef_campaign.rs` — mission spawning, wave progression, boss spawning (pattern after `cg_campaign.rs`)
- Wire into `ElderFleetPlugin`
- 5 missions with progressive difficulty (First Blood → Empire's End)
- Bosses: SquadronLeader (300 HP) → ImperialAdmiral (1200 HP)
- Unit tests for campaign state transitions

### 3. Additional Enemy Variants
Add faction-specific variants for non-Triglavian campaigns:
- `ExecutionerElite` — fast Amarr interceptor with laser beam behavior
- `PunisherTank` — slow, heavily armored Amarr brawler
- `RifterBerserker` — fast Minmatar frigate with autocannon burst
- Wire into Elder Fleet campaign spawning

### 4. Mission-Specific Objective Types
Extend beyond "survive waves / no damage / souls":
- `KillCountObjective` — track enemy kills, display counter on HUD
- `TimedSurvivalObjective` — survive for X seconds, countdown timer
- `EscortObjective` — protect friendly entity that moves along a path
- Store in `Mission` struct, evaluate in campaign systems, display progress in HUD

### 5. Content Completion
- Expand Elder Fleet module.json from 5 missions to full 13 across 3 acts
- Add mission-specific dialogue triggers
- Add act transition screens

## Build Rules
1. Every change must have tests (unit or integration)
2. Run `cargo test && cargo clippy -- -D warnings` before every commit
3. Use conventional commits: `feat:`, `fix:`, `test:`, `docs:`
4. Update `HANDOFF.md` when completing a priority item
5. Integration tests go in `tests/*.rs`; unit tests go inline in source files
6. Headless app pattern: `build_headless_app()` + manual state transitions

## Anti-Patterns to Avoid
- Don't add variants without wiring them into at least one spawn path
- Don't change `GameState` enum without updating ALL `.or(in_state(...))` conditions
- Don't write integration tests that depend on `ButtonInput::just_pressed` auto-clearing in headless mode (it doesn't)
- Don't duplicate boss logic across campaigns — refactor shared patterns into `entities/boss/`

## Session End Checklist
- [ ] `cargo test` passes
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `HANDOFF.md` updated with completed items
- [ ] Conventional commit message explaining what was learned/decided
- [ ] No secrets or credentials in any file
