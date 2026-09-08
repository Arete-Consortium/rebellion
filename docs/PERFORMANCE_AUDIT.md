# Rebellion performance audit

Date: 2026-09-06 (America/Los_Angeles). Source: standalone Rust/Bevy
repository at commit `226f3bc`, plus the current working changes.

This pass establishes specific allocation, particle-budget, and diagnostic
defects from source inspection. It does not establish an FPS improvement,
device performance result, or production rendering benchmark.

## Changes in this working tree

| Change | Previous behavior | Current behavior | Regression coverage |
| --- | --- | --- | --- |
| Enemy spatial lookup | Allocated a nine-entry index vector for every neighborhood query. | Traverses the fixed neighborhood lazily, preserving row order and insertion order within each cell. This retains the first-hit selection behavior. | Exact neighborhood contents, traversal/insertion ordering, clipped corners, padded edge, and queries outside the grid. |
| Bullet trails | Checked the particle cap once before all emitters; every emitter could then queue more particles through its catch-up loop. | Reserves capacity for each queued spawn, including deferred commands. Drops excess emissions while retaining the fractional timer interval. | Crowded 250 ms frame with 100 emitters, existing particles, desktop/mobile caps, zero budget, and no later backlog burst. |
| Engine trails | Did not read `PerfProfile.engine_particles`; spawned two particles for every elapsed emission interval. | Uses the engine particle budget; the last available slot can receive a core particle without its glow. Emission work remains bounded by available capacity. | Same crowded-frame coverage, including the odd mobile engine cap of 25. |
| Boss frame diagnostics | Treated Bevy's millisecond sample as seconds, compared it with `0.020`, then multiplied it by 1000 for output and averages. | Keeps samples and averages in milliseconds, compares with `20.0`, and reports the value directly. | A 16 ms sample remains 16 ms, adding 33 ms averages 24.5 ms, and the rolling window evicts samples beyond its latest 60. |
| Spatial benchmark fixture | Created a new grid inside each iteration and positioned 500 enemies in 16 columns extending beyond grid height. Only 256 entries were retained. | Uses 500 in-bounds enemies, 500 query positions, and retained grid capacity. Separate benchmarks measure queries and rebuild-plus-queries. | Fixture and current-run CPU measurements are recorded below. |

Sources:

- [`src/systems/collision.rs`](../src/systems/collision.rs):
  `SpatialGrid::get_nearby_enemies` and its unit tests.
- [`src/systems/effects/trails.rs`](../src/systems/effects/trails.rs):
  `emission_count`, both trail spawn systems, and their unit tests.
- [`src/diagnostics/mod.rs`](../src/diagnostics/mod.rs):
  `BossFrameProfiler`, `log_boss_frame_time_spikes`, and their unit tests.
- [`benches/game_systems.rs`](../benches/game_systems.rs): `bench_spatial_grid`.
- Bevy `bevy_diagnostic` 0.15.3, `frame_time_diagnostics_plugin.rs`,
  `diagnostic_system`: writes `delta_seconds * 1000.0` to `FRAME_TIME`.
  This was checked in the dependency source used by the current build.

The trail review found no cap-accounting defect in the current changes.
Queued particles consume capacity before commands are applied; odd engine
budgets and zero budgets are handled. Scarce capacity is assigned in query
order, so later emitters may have fewer visible trails. Actual playtesting
must assess the visual result.

## Outstanding findings

### 1. Environment lookup still allocates per projectile

`SpatialGrid::get_nearby_environments` constructs both `results` and `seen`
with capacity 16 on every call, even when no environment objects exist.
`detect_player_projectile_environment_hits` and
`detect_enemy_projectile_environment_hits` each call it for every projectile;
the player contact system also calls it each fixed tick.

The source proves this allocation pattern. It does not prove that it
dominates frame time. A bounded next change could avoid allocation for an
empty environment grid and reuse scratch storage for populated queries.
Preserve entity deduplication, neighborhood traversal order, and first-hit
behavior. Test large objects spanning cells and benchmark both zero-object
and populated cases before expanding the change.

Sources: [`src/systems/collision.rs`](../src/systems/collision.rs),
`get_nearby_environments`; [`src/simulation/detect_collisions.rs`](../src/simulation/detect_collisions.rs),
the three environment detection systems.

### 2. Environment bounds can insert offscreen objects across the grid

`insert_environment` converts a negative upper cell bound to `usize` before
clamping it to the grid's upper bound. For an object centered at
`(-1000, 0)` with radius 20, the upper X cell is `-11`; the conversion and
clamp turn that into column 17. The lower X cell clamps to zero, inserting
the fully offscreen object into all 18 columns across the two Y rows.

This creates unnecessary broad-phase entries and candidates. The narrow
phase still checks actual distances; this audit does not claim that the
extra entries themselves cause false damage. Reject rectangles that do not
intersect the grid and clamp in signed coordinates before conversion.
Regression cases should cover fully outside left/bottom bounds, partially
overlapping bounds, and large objects spanning several cells.

Source: [`src/systems/collision.rs`](../src/systems/collision.rs),
`insert_environment`.

### 3. Explosion cap admits full bursts beyond remaining capacity

`handle_explosion_events` checks whether the current count is below the cap,
then `spawn_explosion_capped` queues a complete size-based burst. With 59
existing particles and a mobile cap of 60, a massive burst queues 50 more,
raising the main-particle count to 109. Embers, flashes, and shockwave rings
are separate entities outside that count.

Pass remaining capacity into the emitter and define an explicit budget for
secondary effects. Preserve the readable explosion flash and separate any
haptic decision from visual capacity. Test nearly full budgets, repeated
events in one update, and zero budgets.

Source: [`src/systems/effects/explosions.rs`](../src/systems/effects/explosions.rs),
`handle_explosion_events` and `spawn_explosion_capped`.

### 4. Mobile profile initialization lacks an ordering dependency

`apply_mobile_profile_if_active` and `detect_mobile_mode` are Startup systems
in different plugins. The former's comment says it runs after detection,
but no system ordering edge enforces that. Source inspection proves the
missing dependency; this pass has not reproduced incorrect initialization
on a touch device.

Add an explicit ordering relationship or initialize the profile from the
same ordered startup chain. Test the effective profile after startup for
mobile and desktop states, then verify detection on an actual device.

Sources: [`src/systems/perf_profile.rs`](../src/systems/perf_profile.rs),
`PerfProfilePlugin::build`; [`src/systems/touch_joystick.rs`](../src/systems/touch_joystick.rs),
`TouchJoystickPlugin::build`.

### 5. Existing performance evidence does not measure rendered frame time

The Mission 3 stress tests check enemy/entity bounds. The environment stress
test checks that 100 objects survive 60 updates; despite its comment, it
does not demonstrate linear scaling. The shared headless app omits the
renderer and the presentation effects plugin. The new isolated trail tests
exercise particle accounting but also omit GPU rendering.

The boss diagnostic consumer reads `smoothed()`, so its warning value is a
smoothed observation rather than the duration of an individual spike. Its
rolling average further averages these observations. Corrected units make
the output meaningful, but it cannot supply raw-frame percentiles or prove
that short hitches are absent.

Sources: [`tests/cg_mission3_stress.rs`](../tests/cg_mission3_stress.rs),
[`tests/environment_collision.rs`](../tests/environment_collision.rs),
[`src/app_builder.rs`](../src/app_builder.rs),
[`src/diagnostics/mod.rs`](../src/diagnostics/mod.rs).

## Validation status

The final working tree passed `cargo test --offline --locked` (506 tests,
zero failures), `cargo clippy --offline --locked --all-targets -- -D warnings`,
formatting, and `git diff --check`. This includes collision/order, trail-budget,
frame-time unit, and instrumentation-gate regressions. The native executable
also initialized Metal and bundled assets and remained alive for a bounded
20-second startup check. Full commands and limits are recorded in
[`COMPLETION_PLAN.md`](COMPLETION_PLAN.md).

No native, desktop-browser, or mobile rendered frame-time comparison has
been completed in this audit.

## Current CPU benchmark observation

Executed on the same macOS/aarch64 host with Rust 1.98.1 and this working
tree, after closing the native smoke-test process:

```sh
cargo bench --offline --locked --profile dev --bench game_systems -- spatial_grid
```

This uses the repository's **dev** profile: application optimization level 1,
dependency optimization level 3, and debug information. It is not a release
build. Each benchmark used a three-second warmup and 100 samples over an
estimated five seconds. Results are for a whole batch of 500 queries:

| Fixture | Criterion central estimate | Reported interval |
| --- | ---: | ---: |
| Query 500 in-bounds enemies from 500 positions | 8.6124 microseconds | 8.5586–8.6647 microseconds |
| Rebuild the retained grid, then run those 500 queries | 11.173 microseconds | 11.123–11.226 microseconds |

Temporary log: `/private/tmp/rebellion-spatial-bench.log`; Criterion reports
are under ignored `target/criterion/`. The benchmark counts neighborhood
entries. It excludes narrow-phase collision tests, damage resolution,
effects and rendering. No comparable pre-change run was made with the
corrected fixture, so these observations establish neither a speedup nor
a rendered frame-rate improvement. Reuse the corrected fixture on both
revisions for a meaningful comparison.

## Measurement plan

1. Run the relevant unit suites and existing projectile/environment
   integration tests, then compile the benchmark target. Record Rust
   version, target, commit and working diff, build profile, and commands.
2. Run the two corrected spatial benchmarks on the same machine and build
   profile. To compare old and new lookup implementations, use the same
   corrected fixtures on both revisions. The historical benchmark fixture
   is not a comparable baseline. These measurements cover CPU lookup and
   rebuild work, not collision resolution, effects, or rendering.
3. Establish a repeatable rendered route through Caldari-Gallente missions
   1–3 and the Mission 3 boss. Include dense projectile patterns, homing
   weapons, mass explosions, environment objects, and transitions. Record
   input/seed and actual entity peaks; repeat the same route in each build.
4. Use release builds and record device, OS, browser, graphics backend,
   viewport, refresh rate, effective profile and power/thermal conditions.
   Warm up asset/shader loading separately. Capture at least three
   equivalent 60-second combat windows per target, including the affected
   iPhone/Safari target and a desktop renderer.
5. Capture raw frame durations and report median, p95, p99, maximum,
   counts over 16.67/33.33 ms, and particle/projectile/entity peaks. Separate
   first-load stalls, steady combat and transition stalls. Record CPU and
   GPU timing separately where supported. A 16.67 ms frame budget for a
   proposed 60 FPS target is a goal, not an observed outcome.
6. Review combat readability while capped: enemy bullet visibility, ship
   identification, explosion feedback and emitter fairness. Accept a
   performance change only with its correctness checks, repeatable timing
   evidence, and acceptable playability on the declared target devices.

Focused commands for future changes:

```bash
cargo test --lib systems::collision::tests
cargo test --lib systems::effects::trails::tests
cargo test --lib diagnostics::tests
cargo test --test projectile_e2e --test environment_collision
cargo bench --bench game_systems --no-run
cargo bench --bench game_systems -- spatial_grid
```
