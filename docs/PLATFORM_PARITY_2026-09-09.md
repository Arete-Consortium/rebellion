# Matching platform playtests — September 9, 2026

The owner's requirement is that all versions have similar content and mechanics.
Mac, Linux, Windows and browser packages now use the same game source, campaign
configuration, controller layout, balancing, run statistics and runtime assets.
Storage, graphics, windowing and controller backends remain platform adapters.
Each candidate must still pass its own startup and physical-device checks.

## Candidate and evidence

Source candidate: `6bf8ca15c93a685c257956dcf1b816815833fbac` on
`codex/linux-playtest`. The [Platform Playtest run](https://github.com/Arete-Consortium/rebellion/actions/runs/34422890302)
completed successfully for all four platforms. Its **Verify package parity**
job passed, and independent verification of the downloaded archives matched the
CI report exactly. All builds used Rust 1.98.1 and the committed dependency lock.

| Platform | Matched download | Startup evidence |
| --- | --- | --- |
| Apple Silicon Mac | [Mac artifact](https://github.com/Arete-Consortium/rebellion/actions/runs/34422890302/artifacts/10131896366) | Extracted signed app, 132 assets, isolated LaunchServices startup for 12 seconds; passed |
| Intel/AMD Ubuntu 22.04+ | [Linux artifact](https://github.com/Arete-Consortium/rebellion/actions/runs/34422890302/artifacts/10131782731) | 586 tests plus extracted-package Xvfb/software-Vulkan startup; passed |
| Intel/AMD Windows | [Windows artifact](https://github.com/Arete-Consortium/rebellion/actions/runs/34422890302/artifacts/10132046587) | Native Windows extracted-package startup, isolated save and checksums; passed |
| Browser | [Web artifact](https://github.com/Arete-Consortium/rebellion/actions/runs/34422890302/artifacts/10131820982) | Chrome 152 on Mac: controller fixture, menus, combat, pause/reconnect, death/results and storage reload; passed |

GitHub sign-in is required. These artifacts expire October 10, 2026 UTC. The
[parity report](https://github.com/Arete-Consortium/rebellion/actions/runs/34422890302/artifacts/10132068730)
contains exact archive and content hashes. Local copies are under
`dist/platform-parity-20260909/`; the extracted Mac app is
`dist/platform-parity-20260909/macos/Rebellion.app`.

The original workspace remains on `codex/finishability-pass` with its earlier
staged, unstaged and untracked work. The candidate was reconstructed and committed
in a separate checkout, with 366 source, test, asset, packaging and document files
byte-verified against the workspace. `main` and public release/site publication
were not part of this playtest update.

Shared source fingerprint:
`f7eb7c4cc2337f7d9a59040c1137ace02ae2c1524a4c7d543a45f5814408d0e5`

Asset fingerprint (132 files):
`5ca63f81f99e85432b9c90591c5f636017e96f7e22517a6db185f011ac36463e`

Local evidence: `build/playtest-review/platform-parity-20260909/`.
The complete local gameplay suite passed **586 tests**, with zero failures or
ignored tests across 40 target summaries. Formatting and all-target Clippy with
warnings denied passed. Package-manifest regression checks passed **18 tests**.
The separate startup results above verify the actual packaged executables and
browser bundle. The browser test used a test-only standard Gamepad API fixture,
with ordinary controls and enemy damage. It earned 82 points, reached the death
screen, retained a best chain of 1 and 19.2 seconds of combat time, preserved
pause through reconnect, and retained that exact record after reload. The long
pause did not inflate combat time. It does not prove physical-device detection,
sound, rumble or a complete human playthrough.

Evidence files include `test-result.json`, `clippy.log`, `browser-result.json`,
`browser-persistence-result.log`, `output/playwright/`,
`mac-smoke/bundle-manifest.json` and `downloaded-package-parity/result.json`.
CI Windows/Linux startup reports are retained with the downloaded artifacts.
Browser logs include nonfatal optional `.meta`/favicon HTTP 404s and verbose wave
transition messages; the actual ship load completed 80/80 with zero failed sprites. No
fatal browser error or missing required asset was observed.

## Shared behavior

- Both Minmatar–Amarr and Caldari–Gallente chapters are exposed on every target.
  The finishing target remains the Caldari/Gallente three-mission slice; wider
  content does not become release-qualified merely by being included.
- Controller-only navigation and play: left stick moves, right stick aims and
  fires, RT thrusts, LT uses the hull ability, RB dodges, LB uses the charged
  overload, and A confirms/interacts. Menu pauses. The same neutral/A/release
  connection gate freezes simulation during a disconnect.
- D-pad up/down selects a stored timed booster and Y spends one dose per fresh
  press. Booster effects, capacity, mission carryover and reset rules match.
- Results retain peak chain and combat time, and terminal results compare with
  the preceding personal best. Ordinary pause/mission continuation retains the
  run; a new attempt resets it. Save schemas are shared across disk and browser
  storage, though those stores remain separate.

Browser builds previously defaulted to a legacy preview mode. That default is
now disabled on every platform, retaining explicit opt-in legacy behavior only.
The browser shell now exposes the game's controller gate directly, preserves
real startup errors, and tests WebGL capability using a separate canvas.

## Packaging and parity gate

Use all platform packages from the same successful **Platform Playtest** run.
The artifact set is `Rebellion-macos-arm64`, `Rebellion-linux-x86_64`,
`Rebellion-windows-x86_64` and `Rebellion-web`. See the
[Linux](LINUX_PLAYTEST.md), [Windows](WINDOWS_PLAYTEST.md) and
[browser](WEB_PLAYTEST.md) guides for extraction and startup.

Every package contains `PLAYTEST-MANIFEST.json`: inside `Contents/Resources`
for Mac, or at the package root for the other targets. It records the Git
revision, clean/modified source state, shared source/configuration hashes and
the inventory of packaged assets. The final CI job opens the actual archives,
verifies their assets, and requires one clean source commit and identical source
and asset fingerprints across all four platforms. Its report is uploaded as
`Rebellion-platform-parity`. The gate must pass before calling the set matched.

Windows packaging uses the static MSVC runtime and checks its extracted
executable with an isolated muted save. Linux retains the extracted-package
Xvfb/software-Vulkan startup check. Mac packages are ad-hoc signed and verified.
The web packager uses the exact `wasm-bindgen` version from `Cargo.lock`, the
`wasm-release` profile, fresh assets and the canonical HTML shell. CI, Pages and
release browser builds share that build script. Packaging refuses existing
destinations, preserving previous working candidates.

## Remaining acceptance

Record physical-controller connection/reconnect, sound, rumble, device
performance, pause/death/retry, mission continuation and complete human routes
for each claimed platform. Browser acceptance still includes other named
browsers, resize/focus and audio activation. Mobile remains a separate device
qualification; a browser compile does not create a touch version of the game.

The prior [run-results checkpoint](RUN_RESULTS_PLAYTEST_2026-09-09.md) and
[manual-booster checkpoint](MANUAL_BOOSTER_PLAYTEST_2026-09-08.md) retain historical
candidate downloads and evidence. Their archives are not updated in place.
