# Mission 2 Candidates

**Generated:** 2026-07-12  
**Source:** PR #9 (`arch/m01-collision-split`) observation + existing audit findings  
**Rule:** Every item here was noticed during Mission 1 but intentionally not fixed to keep Mission 1 scope to "plugin boundaries only, no gameplay behavior change."

---

## Determinism & Timestep

| ID | Issue | Location | Severity | Notes |
|----|-------|----------|----------|-------|
| M2-001 | No `FixedUpdate` / `SimSet` | Entire codebase | P0 | All gameplay runs in frame-based `Update`. Combat timing varies by FPS. Needs `Time<Fixed>` migration. |
| M2-004 | Frame-rate dependent cooldowns | `combat_reactions.rs`, assumed across systems | P1 | `last_callout` and similar timers use `time.delta_secs()`. Cosmetic ok, gameplay-affecting is not. |

## RNG Seeding & Separation

| ID | Issue | Location | Severity | Notes |
|----|-------|----------|----------|-------|
| M2-002 | Unseeded `fastrand` in crit/drops | `resolve_damage.rs` | P0 | Critical hit roll and powerup drop chance use global `fastrand::f32()`. Needs `SimulationRng` seeded from `MissionSeed`. |
| M2-003 | Chain bolt jitter uses gameplay RNG | `combat_reactions.rs` | P2 | Bolt zigzag is cosmetic; can use separate `PresentationRng`. |
| M2-013 | `ContactDetected` carries raw crit_chance/ammo_type | `core/events.rs` | P2 | Detection event contains resolution-level fields (crit_chance, ammo_type). Could split into `ContactRaw` + `ContactResolved`. |

## Architecture Debt

| ID | Issue | Location | Severity | Notes |
|----|-------|----------|----------|-------|
| M2-006 | Duplicate score authority | `GameplayPlugin` | P1 | `ScoringPlugin` + `ScoringSystemPlugin` both registered. Co-located in Mission 1; needs unified design in Mission 2. |
| M2-008 | `main.rs` initializes 11+ resources manually | `main.rs` | P2 | Resources should be self-registered by their owning plugins. |
| M2-009 | No headless app constructor | Missing | P2 | Needed for deterministic simulation tests without a window. |
| M2-010 | No `SimId` stable IDs | Missing | P2 | Needed for replay recording and cross-session entity tracking. |
| M2-014 | `BossCalloutSent` uses component marker | `combat_reactions.rs` | P3 | Works for single-boss case; revisit if multi-boss fights are added. |

## Replay & State Integrity

| ID | Issue | Location | Severity | Notes |
|----|-------|----------|----------|-------|
| M2-011 | No replay recording | Missing | P2 | Needed for bug reproduction and competitive integrity. |
| M2-012 | No state hashing | Missing | P2 | Needed for desync detection in replay verification. |

## Campaign/Data

| ID | Issue | Location | Severity | Notes |
|----|-------|----------|----------|-------|
| M2-007 | Campaign plugins executable | `games/` | P3 | Campaigns are code plugins (`ElderFleetPlugin`, etc.). Mission 4+ will convert to RON data. |

---

*Append-only during Mission 2 planning. Items may be promoted, demoted, or split into sub-items as design matures.*
