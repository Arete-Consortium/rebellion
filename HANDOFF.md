# Rebellion handoff — September 9, 2026

The finishing target is a complete **Caldari–Gallente three-mission slice**, with both Minmatar–Amarr and Caldari–Gallente chapters exposed for playtesting. The [production plan](docs/PRODUCTION_PLAN.md) owns priorities and release gates. Start with the [platform parity checkpoint](docs/PLATFORM_PARITY_2026-09-09.md) for the matching Mac, Linux, Windows and browser candidate, verification state and remaining acceptance. The owner requires similar content and mechanics on all versions.

## Source and candidate state

- Matching candidate source: `6bf8ca15c93a685c257956dcf1b816815833fbac` on `codex/linux-playtest`. [Platform Playtest run 34422890302](https://github.com/Arete-Consortium/rebellion/actions/runs/34422890302) succeeded for Apple Silicon Mac, Ubuntu x86_64, Windows x86_64 and browser packages. The final gate verified one clean source commit and identical source/configuration plus all 132 packaged assets.
- Local packages: `dist/platform-parity-20260909/`. The extracted Mac app is `macos/Rebellion.app`; downloadable archives and CI startup/parity reports are in `artifacts/`. Browser files are in `web/Rebellion-web/`.
- Fresh validation: 586 gameplay tests on Mac and Linux, 18 package-manifest tests, formatting and all-target Clippy. The Mac package passed signature/assets and isolated LaunchServices startup; Windows and Linux passed native extracted-package startup checks. The compiled browser game passed controller-fixture navigation, combat, pause/reconnect, natural death/results and score persistence after reload in Chrome 152.
- Source and detailed evidence: [platform checkpoint](docs/PLATFORM_PARITY_2026-09-09.md), with local reports at `build/playtest-review/platform-parity-20260909/`. Physical-controller detection, feel, sound, rumble, hardware performance and complete human playthroughs remain unqualified.
- The original workspace remains on `codex/finishability-pass`, HEAD `226f3bc`, with extensive earlier staged, unstaged and untracked work. It was not switched or committed wholesale; candidate publication used a separate checkout. Its branch name and HEAD alone do not identify its current game content.

The [earlier run-results checkpoint](docs/RUN_RESULTS_PLAYTEST_2026-09-09.md) and [manual-booster checkpoint](docs/MANUAL_BOOSTER_PLAYTEST_2026-09-08.md) retain the preceding local Mac and Mac/Linux candidates. Their old archives remain historical evidence; use the platform checkpoint for current downloads.

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
