# Run results checkpoint — September 9

This follow-up implements RBL-06 from the [production plan](PRODUCTION_PLAN.md). It builds on the [manual booster candidate](MANUAL_BOOSTER_PLAYTEST_2026-09-08.md) and keeps the current controller and booster behavior.

## Player behavior

- Mission and terminal results retain the attempt's **best chain** after the live combo expires. Combat time includes ordinary waves and boss fights; loading, briefing, boss introductions, pause, disconnect freezes and result screens do not add time.
- Mission continuation preserves score and attempt statistics, while starting the next mission clears the live combo. Pause/resume preserves both. Returning to the main menu, selecting a hull, death retry and explicit restart clear the attempt. Restart still uses the currently selected mission.
- Death, slice completion and campaign victory compare against the existing faction-pair personal best before recording the result. Results show the improvement, a tie, the points needed to match the best, or the first scored run. Entering a terminal result again cannot record the same attempt twice.
- A strictly better score stores its chain, peak multiplier and combat time together. Lower scores and ties preserve the previous record and its statistics. Existing saves remain readable; older records retain unknown statistics until beaten.
- Caldari/Gallente death and slice records use the same keys as their existing campaign-victory records. Elder Fleet records the actual mission reached, including mission nine. Hidden modes keep their existing persistence behavior.

This also fixes old scores leaking into a new campaign after exiting to the main menu. The death screen now gives controller-only prompts in player builds. Results use separators supported by the shipped font.

## Verification and artifacts

Verification logs, the focused change, frozen input manifest, native captures and package qualification are kept under `build/playtest-review/run-results-20260909/`. The native fixture uses synthetic statistics and an isolated muted save; its captures establish layout and displayed comparisons, not a human campaign completion.

The local source remains on the existing dirty `codex/finishability-pass` checkout. Its HEAD is not the candidate's content identity; `source-manifest.json` records the exact inputs. This checkpoint is separate from the published September 8 Linux candidate.

The final full suite passed **585 tests**, with zero failures or ignored tests across 40 target summaries. Formatting, all-target Clippy with warnings denied, and staged/unstaged whitespace checks passed. Nine new lifecycle/persistence tests and six result-screen tests cover the added behavior. The lifecycle harness uses in-memory saves; the full suite uses an isolated profile.

The native fixture captured mission, slice, death and Elder Fleet victory screens at 800×700 logical resolution, verified the comparison labels and exited successfully. All four initial layouts were inspected; the corrected death/slice captures confirm supported separators and controller prompts. The initial sandbox launch lacked GPU access; the successful review ran with normal Mac GPU access. Existing B0003 menu-cleanup warnings and the transitive `block 0.1.6` future-compatibility notice remain.

The candidate manifest freezes **346** source, test, asset, configuration and packaging inputs. All remained unchanged through packaging and startup qualification.

**Apple Silicon Mac:** `dist/run-results-20260909/Rebellion.app`, with `Rebellion.app.zip` beside it. A fresh extraction passed strict signature verification and exact bundle/asset hash comparisons. Normal LaunchServices startup from an unrelated directory initialized successfully, remained alive for twelve seconds, and produced no panic or missing-asset error. The owned test instance closed cleanly. An initial attempt to redirect launch logs into the workspace failed before startup; the repeated qualification captured logs beside the temporary extraction and copied the evidence back.

- App: 57,872,050 bytes across 136 files; ZIP: 32,140,812 bytes.
- Executable SHA-256: `a8d6ab3ef6de6485e514d1706cfc9b4fb881e4608d5e70f594a19e276fae740e`.
- ZIP SHA-256: `4927a24eef69c9bdd1a25cbdbb2ebd4e805030a4d675122efe6fac883af24e58`.
- `bundle-manifest.json`, `package.log`, `qualification.log` and `packaged-startup.log` record the checks; the failed first attempt is preserved separately.

Rust 1.98.1, rustfmt, Clippy and locked dependencies were restored under `/private/tmp/rebellion-rust-bootstrap/` because the prior temporary installation was gone. Shell startup files were not changed. While this temporary installation remains available, verification commands use:

```sh
export CARGO_HOME=/private/tmp/rebellion-rust-bootstrap/cargo
export RUSTUP_HOME=/private/tmp/rebellion-rust-bootstrap/rustup
export PATH="$CARGO_HOME/bin:$PATH"
```

## Next playtest

On the physical controller, finish a mission, let a chain expire, pause and resume, then die/retry or complete the slice. Check that the best chain remains visible, time excludes pauses, a new attempt starts at zero, and a personal best survives relaunch. The [controller playtest](CONTROLLER_PLAYTEST.md) still owns hardware, stick feel, rumble and complete human route acceptance.

The Linux guide and current handoff were reconciled with controller-only twin-stick play and stored boosters. Existing Linux archives keep their original bundled instructions and game code until rebuilt. A Linux candidate containing these run-result changes remains to be built and qualified.
