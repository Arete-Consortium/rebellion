# Rebellion completion plan

Working assessment: 2026-09-06. Source baseline: `226f3bcdf5778cde60927e5d720c6fdc91de6f64` in `Arete-Consortium/rebellion`. Local work branch: `codex/finishability-pass`.

The [production plan](PRODUCTION_PLAN.md) is the active roadmap and backlog.
This document preserves the first repair pass and its verification evidence.

## Finish the first playable release

The existing target is the Caldari–Gallente three-mission vertical slice. Keep that scope while proving the whole route works: menu → faction/ship choice → waves → bosses → mission results → slice results → replay. The other campaigns already contain substantial implementation, but each needs equivalent completion evidence before joining the release scope.

Desktop is the provisional first validation target, using this Mac. Browser support remains in scope as a later build/QA pass; this is a working assumption pending the owner's platform preference.

The game should earn its quality through responsive movement, readable threats, satisfying weapons, and replayable encounters. Use faction-specific weapons and ship handling to preserve its EVE identity. A larger feature list is not a completion criterion.

## First repair pass

| Problem found | Change | Verification surface |
| --- | --- | --- |
| CG wave delay re-arms before wave two can spawn | Spawn on the expiry update; share actual enemy count with warnings | `tests/cg_campaign_progression.rs`: production check-before-spawn schedule, wave progression, 120 Hz warnings |
| Simulation despawns CG boss before campaign observes its death | Consume the simulation's destruction event | Actual projectile detection → damage → death → campaign completion regression |
| Pause/resume re-enters mission initialization and loses combat state | Preserve the prior combat state and guard entry/exit lifecycle hooks | Pause/resume regression coverage; native input smoke still required |
| Every enemy-neighborhood lookup allocates a temporary vector | Traverse the nine possible cells with an iterator preserving contact order | Neighborhood/order/boundary tests and corrected spatial benchmark |
| Bullet trails exceed their cap; engine cap is unused | Reserve capacity including deferred spawns and discard excess emission debt | Crowded 250 ms frame tests for desktop/mobile, zero limits, existing particles, backlog |
| Frame-time diagnostics interpret milliseconds as seconds | Keep the measurement, average and spike threshold in milliseconds | Known frame-time samples and rolling-window tests |
| Root README describes only ship-registry tooling | Restore game setup/controls/verification documentation; preserve old README in `docs/SHIP_REGISTRY_PACKAGE.md` | Commands checked against source and build configuration |
| Native menu systems panic during query initialization | Make Options queries disjoint and identify actual slider fills explicitly | Native menu and full game-system schedule initialization; actual Options widget behavior |
| Horizontal Options input changes rows; Controls captures its activation button | Reserve horizontal input for slider changes and ignore initially held capture buttons until release | Actual native handlers, stick/D-pad/keys, same-update activation and release/repress regressions |

The deeper code/materials review also removes confirmed duplicate work:

- Move 38,349,930 bytes of review snapshots/contact sheets and 2,461,819 bytes of editable STL sources/conversion tooling out of runtime `assets/`. The asset tree decreased from 54,516,725 to 13,704,976 bytes (about 75%). Existing package copy commands therefore stop shipping those materials. Runtime GLBs and ship sprites remain unchanged.
- Deduplicate 147 byte-identical review files (7,023,225 bytes), retaining every unique variant and metadata file. The archive's hash-checked index reconstructs all 588 originals from 441 stored files.
- Remove 1,162,054 bytes of reproducible audit output and the accidental empty `=` file; keep new generated output under ignored build paths. Git history remains intact.
- Retain the configurable UI starfield and remove the second, 150-star effects implementation.
- Make SimId/state hashing opt-in for native/WASM play, while enabling it for headless regression tests.
- Remove unused direct Tokio/Reqwest JSON features and empty collision-plugin/CG mission-update scaffolding.

Read [CODE_REVIEW.md](CODE_REVIEW.md), [GAMEPLAY_REVIEW.md](GAMEPLAY_REVIEW.md), and the [ship-review archive](ship-review/README.md) for evidence and remaining work. Desired campaigns and unique source/review art were preserved.

Verification results are recorded below. Changes remain local on the working branch; no commit, remote push or published build was created.

## Next production work

Follow the milestones, release contract and first-sprint backlog in
[PRODUCTION_PLAN.md](PRODUCTION_PLAN.md). The first checkpoint is a packaged
native Caldari–Gallente playthrough with an isolated save, followed by repairs
to remaining input, persistence and route defects. Keeping the live backlog
there avoids maintaining two competing milestone lists.

## Known gaps to keep visible

- Existing headless tests omit much of the native menu/presentation stack. Some tests call spawning manually and previously missed the actual schedule-order bugs. Passing the suite does not prove a human can finish the game.
- The existing slice document describes 20–30 minutes, while newer inter-wave timer comments target 8–12 minutes. Measure an actual run and choose a single intended duration before tuning pacing. Do not silently rewrite the original scope document.
- The browser build script installs the latest binding CLI if absent; CI pins a specific CLI version. Match the CLI to the lockfile and verify the browser artifact before distribution.
- Particle limits are structural bounds, not measured FPS improvements. Scarce capacity can favor earlier emitters; inspect how the reduced effects look in motion.
- Environment-grid allocation/bounds behavior, explosion limits and mobile-profile startup ordering remain candidates described in `PERFORMANCE_AUDIT.md`.
- Direct image review found screenshot backgrounds, oblique views and cartoon placeholders mixed with clean cutouts in the preloaded runtime sprites. The loader trusts bundled transparency; the normalization archive is not an approved replacement. See the materials findings in `CODE_REVIEW.md` before treating artwork as final.
- Repository description, package version, menu version and older handoff documents disagree. This file identifies the inspected source revision; reconcile release metadata when preparing the actual release.

## Verification record

Validated the final working tree on macOS, `aarch64-apple-darwin`, using Rust
1.98.1 (`48a229cea`, 2026-09-01). The manifest's declared minimum Rust 1.85
was not independently tested. The lockfile changed only to remove unused
dependency edges and `tokio-macros`; dependency versions were not upgraded.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo test --offline --locked` | **506 passed, zero failed or ignored**, across 34 reported suites (including zero-test targets) |
| `cargo clippy --offline --locked --all-targets -- -D warnings` | Passed; Cargo separately reports an existing future-compatibility warning in transitive `block` 0.1.6 |
| `cargo bench --offline --locked --profile dev --bench game_systems -- spatial_grid` | Passed: 500-query estimate 8.6124 microseconds; rebuild plus queries 11.173 microseconds. Dev profile, no comparable before-run or FPS claim; details in `PERFORMANCE_AUDIT.md` |
| `python3 scripts/materialize_ship_review.py --verify-only` | Passed: 588 original file identities backed by 441 verified stored files |
| Archive reconstruction and STL/GLB preservation | All 588 reconstructed files and all 11 STL/11 GLB files matched their original Git blobs; conflicting recovery output was refused |
| `git diff --check` and links in the new/updated review documents | Passed |
| Native startup of rebuilt debug executable | Metal initialized on the adapter reported as Apple A18 Pro; window created; 48 bundled sprites loaded; procedural audio generated; remained running for the 20-second check and was closed by the test harness |

The final suite includes nine pause/resume/restart/quit tests, five CG
progression tests using real scheduling and collision/death systems, native
Options/Controls handler regressions, full native system-query initialization,
trail-budget tests and diagnostics-gating coverage.

Native startup still logs five missing optional music overrides under
`assets/audio/music/*.ogg`; the procedural audio path initializes. This is
documented in `CODE_REVIEW.md`. No claim is made that audio was listened to,
the entire game was visually inspected, or a human completed a campaign.
The unbundled native executable was not exposed as a controllable app by
the available UI automation surface, so the startup check used process and
renderer/asset logs.

The Rust toolchain was installed in a temporary task directory, without
changing shell startup settings. To reproduce this task's checks while it
remains available:

```sh
export CARGO_HOME=/private/tmp/rebellion-rust-bootstrap/cargo
export RUSTUP_HOME=/private/tmp/rebellion-rust-bootstrap/rustup
export PATH="$CARGO_HOME/bin:$PATH"
export REBELLION_HOME=/private/tmp/rebellion-test-save
cargo test --offline --locked
```

Temporary logs: `/private/tmp/rebellion-final-test.log`,
`/private/tmp/rebellion-final-clippy.log`, and
`/private/tmp/rebellion-native-final-smoke.log`. The native check used a
separate `/private/tmp/rebellion-native-final-smoke` save directory and an
explicit `BEVY_ASSET_ROOT` pointing to this repository.

Unverified release surfaces: Windows/Linux builds, the browser bundle,
controller hardware, complete campaign/save/relaunch routes, clean install
packaging, and rendered frame-time comparisons. The CPU spatial benchmark
is recorded separately in `PERFORMANCE_AUDIT.md`; it cannot establish FPS.
