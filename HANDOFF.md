# Rebellion handoff — September 9, 2026

The finishing target is a complete **Caldari–Gallente three-mission slice**, with both Minmatar–Amarr and Caldari–Gallente chapters exposed for playtesting. The [production plan](docs/PRODUCTION_PLAN.md) owns priorities and release gates. Start with the [run-results checkpoint](docs/RUN_RESULTS_PLAYTEST_2026-09-09.md) for current source and the local Mac candidate. The [manual booster checkpoint](docs/MANUAL_BOOSTER_PLAYTEST_2026-09-08.md) retains the latest Linux candidate and preceding Mac evidence.

## Source and candidate state

September 9 follow-up: [run results and personal bests](docs/RUN_RESULTS_PLAYTEST_2026-09-09.md) implements retained chain/time, terminal records and new-attempt resets. Its local Mac candidate and fresh validation are separate from the September 8 Mac/Linux candidate described below. Start with this follow-up for current source changes.

- Local checkout observed September 9: `codex/finishability-pass`, HEAD `226f3bc`, with extensive existing staged, unstaged and untracked work. Preserve those changes; the local branch name and HEAD alone do not identify the candidate's game content.
- Recorded candidate source: `80f311f4d51ca6d2b283bb15c8370c3241fad3e6` on `codex/linux-playtest`. Its checkpoint records 338 frozen source, test, asset and packaging inputs matching that Git tree.
- Apple Silicon Mac: `dist/manual-boosters-20260908/Rebellion.app` and the adjacent `Rebellion.app.zip`.
- Intel/AMD Ubuntu x86_64: the [recorded candidate download](https://github.com/Arete-Consortium/rebellion/actions/runs/34306848068/artifacts/10087123527). GitHub sign-in is required; the recorded expiration is October 9, 2026 UTC. Use the [Linux instructions](docs/LINUX_PLAYTEST.md) for extraction, dependencies and isolated saves.

The candidate checkpoint records **570 passing tests**, formatting and all-target Clippy with warnings denied; native HUD/controller-guide captures; extracted Mac startup/signature checks; and Ubuntu 22.04 extracted-package checks under Xvfb/software Vulkan. These are September 8 checkpoint results, not a fresh verification of every change in the current working tree. Evidence lives in `build/playtest-review/manual-boosters-20260908/`, including source manifests, `linux-ci-verification.json`, package logs and captures.

Scripted controller pilots completed all three missions for Caldari/Kestrel and Gallente/Tristan with ordinary right-stick fire and no forced kills. Content/drop randomness was unseeded. Physical controller detection, feel, sound, rumble, hardware performance and complete human playthroughs remain unqualified. The longer Tristan boss encounters remain a human pacing check.

## Current player behavior

Controller-only: left stick moves; right stick aims and fires, centering stops primary fire; RT thrusts; LT activates the hull ability; RB dodges; LB uses a charged overload. X/B or D-pad left/right cycles autocannon ammunition. A confirms/interacts, B goes back in menus, and Menu pauses/resumes.

Timed boosters store up to three doses each of Overclocker, Pyrolancea and X-Instinct. D-pad up/down selects an occupied slot; Y uses one dose per fresh press. Empty/already-active selections spend nothing. Doses carry between missions and through pause; death, a new run/hull selection or explicit restart clears them. Repairs, capacitor and cooling pickups remain immediate. Approved booster artwork and combat statistics are unchanged by the manual-use checkpoint.

Startup and reconnect require neutral controls followed by pressing and releasing A. Disconnect freezes simulation; an existing pause menu stays paused after acknowledgement. Keyboard, pointer and touch cannot play or navigate player builds. The [controller guide](docs/CONTROLLER_PLAYTEST.md) is the physical-device acceptance checklist.

## Resume work

1. Inspect the current diff and the relevant checkpoint before changing files. Use a separate checkout for candidate reconstruction; do not switch this dirty workspace to the candidate branch.
2. Qualify the packaged route on the actual Mac and Ubuntu PC: controller connection/reconnect, all controls, stored boosters while dodging, sound, pause/death/retry, mission continuation and final results. Record platform, controller, hull, candidate source, outcome and reproduction steps for failures.
3. Repair concrete blockers and run checks appropriate to each change. Full gameplay verification uses an isolated save:

```sh
cargo fmt --all -- --check
REBELLION_HOME="$(mktemp -d /tmp/rebellion-tests.XXXXXX)" cargo test --offline --locked --no-fail-fast
REBELLION_HOME="$(mktemp -d /tmp/rebellion-clippy.XXXXXX)" cargo clippy --offline --locked --all-targets -- -D warnings
```

4. Build into a fresh output directory and tie new candidate evidence to the exact source. The [Linux guide](docs/LINUX_PLAYTEST.md) and `scripts/package-macos-playtest.sh` describe packaging. Documentation edits do not update existing archives; regenerate packages before distributing revised bundled instructions.

The production plan keeps public release qualification open. Wider campaigns and platform support require their own acceptance evidence.

## Earlier work

The [July 23 campaign sprint handoff](docs/HANDOFF_2026-07-23.md) is preserved unchanged as historical context. Its branch, commands, input bindings and test counts describe that older work. September checkpoints linked from the [README](README.md) record the later chapter, save/audio, transport, booster, hull-ability and controller changes.
