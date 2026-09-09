# Controller-only checkpoint — September 8

The owner selected controller-only play for Rebellion. This checkpoint follows the [hull ability fixes](HULL_ABILITY_PLAYTEST_2026-09-08.md) and makes that input direction explicit in menus, combat and connection handling.

## Player behavior

- Left stick moves with analog strength; right stick aims and retains the last direction when centered. Hold RT to fire.
- LT activates the hull ability. LB thrusts; RB dodges toward the left stick's horizontal direction. Each requires a fresh press, so held controls cannot repeatedly spend capacitor.
- D-pad left/right cycles autocannon ammunition without steering. A interacts/confirms, Y activates a charged overload, B goes back and Menu pauses/resumes. View does not pause.
- The native input boundary suppresses keyboard, pointer and touch gameplay/menu input. Existing saved keyboard mappings cannot override the fixed controller layout; their stored data remains intact for development fixtures.
- Options → Controls now shows one compact, fixed controller guide. Menu hints and the HUD's LT ability label agree with the runtime controls.
- Startup and device replacement require neutral controls followed by an A press and release. The acknowledgement is consumed. Disconnect freezes simulation before another fixed damage tick and requires the same acknowledgement to continue. An already-paused game remains paused.
- The selected controller stays selected while present. A second device cannot silently replace it.

The connection screen uses the game's dark space palette, cyan title and one current action. It covers the frozen game; the controller guide uses aligned rows and no scrolling or capture dialog. Artwork and combat balance are unchanged from the hull ability checkpoint.

## Verification

**561 tests pass**, zero failures or ignored tests across 39 target summaries. All-target Clippy with warnings denied, formatting and diff checks pass. Ten new controller integration tests use actual Bevy `Gamepad` components and the shipped input boundary. They cover absent/held controllers, consumed acknowledgement, both combat states, raw trigger reporting, analog movement, old bindings, both chapters/all four factions, the guide/back route, pause preservation and device replacement.

The native rendering fixture captured the missing-controller screen and the controller guide at the normal 800×700 logical window size. Review caught and corrected the loading title showing through the initial translucent connection overlay; the final screen is opaque. The fixture exits normally with no panic or missing-asset errors. Existing B0003 menu-cleanup warnings remain. The existing `block 0.1.6` dependency notice is unchanged.

The controller-driven campaign probe reached **SliceComplete** on both sides through actual controller menu navigation, ordinary movement/aim/fire, production combat systems and fresh isolated saves. There were no forced kills, altered combat stats or retries. Both boss encounters show declining health and destruction events.

| Route | Mission 1 | Mission 2 | Mission 3 | Active combat total |
| --- | ---: | ---: | ---: | ---: |
| Caldari / Kestrel | 69.2 s | 112.0 s | 132.9 s | 5 min 14 s |
| Gallente / Tristan | 80.1 s | 240.0 s | 122.8 s | 7 min 23 s |

Boss encounters lasted 42.7 / 60.4 seconds for Kestrel and 133.2 / 29.5 seconds for Tristan. Content/drop randomness remains unseeded. These are route observations, not controlled hull-DPS or human-difficulty comparisons.

The source is on `codex/linux-playtest` at `1c828bd21362cdafe407db17fd65383e7bbc6489`. All 336 frozen source, asset, test and packaging inputs match that Git tree and are recorded in `source-manifest.json`. Assets are unchanged from the preceding checkpoint. The full 561-test suite and all-target Clippy passed again after the final corrections, using a fresh writable save directory.

The [first Linux attempt](https://github.com/Arete-Consortium/rebellion/actions/runs/34302564458) exposed test-profile leakage: the controller suite's deliberately remapped legacy Fire binding was saved into the directory later used by hull-ability tests. A local sequential reproduction confirmed the same three failures. The controller suite now uses a private process-specific save directory; the shared profile stays untouched and all eleven subsequent hull-ability tests pass. The fix changes test isolation, not the game binary.

Evidence directory: `build/playtest-review/controller-only-20260908/`. Before files preserve the prior checkpoint for a focused diff. Development examples inject explicitly named synthetic controllers; those helpers are not included in packaged executables. The campaign probe's `--controller` mode uses gamepad components and controller menu navigation instead of keyboard confirmation or direct selection changes.

```sh
cargo run --offline --locked --example campaign_probe -- build/playtest-review/my-controller-probe --controller
```

## Packages

The final Apple Silicon Mac candidate is `dist/controller-only-20260908-r2/Rebellion.app`, with `Rebellion.app.zip` beside it. A fresh ZIP extraction under `/private/tmp` passed strict signature verification and launched through normal macOS LaunchServices from an unrelated working directory. It remained alive for the twelve-second startup check, using an isolated muted save, with no panic or missing assets. The test instance was then closed. The earlier candidate remains preserved.

- App: 57,789,570 bytes, 136 files. ZIP: 32,118,685 bytes.
- Executable SHA-256: `f7a72f2d1730d9eea22a4cb458059367790554d8fb849489e66d84029a3716a2`.
- ZIP SHA-256: `22359f2fdd6d13edeedcf3e2d9d636c6362adfda5452ff17d91f32966efa08dd`.
- Extracted executable and all assets match the candidate; all 336 frozen inputs remained unchanged. `bundle-manifest-final.json`, `package-final.log` and `packaged-startup-final.log` record the checks.

**Ubuntu / Linux Intel/AMD x86_64:** [download the tested package](https://github.com/Arete-Consortium/rebellion/actions/runs/34303199529/artifacts/10085884913) — 35,870,212 bytes (about 36 MB). GitHub sign-in is required. This Actions artifact expires October 9, 2026 UTC. Follow the [Linux instructions](LINUX_PLAYTEST.md) to extract and run it.

The corrected [Ubuntu 22.04 build](https://github.com/Arete-Consortium/rebellion/actions/runs/34303199529) passed all **561 tests**, zero failures or ignored tests across 39 target summaries, using Rust 1.98.1, then built the optimized release. A fresh package extraction passed checksums and shared-library resolution, initialized from an unrelated working directory with an isolated muted save, and stayed alive for the startup check with no panic or missing assets. CI used Xvfb and software Vulkan. The [startup evidence](https://github.com/Arete-Consortium/rebellion/actions/runs/34303199529/artifacts/10085885267) includes its log and machine-readable result.

The Linux artifact records source `1c828bd21362cdafe407db17fd65383e7bbc6489`, matching the Mac candidate's 336 frozen inputs. GitHub artifact digest: `sha256:e1cda6d0f8dc0a9eb7a25d68a01a13d1554d612e115a30fc082418b403131279`. The included tarball checksum is separate. `linux-ci-verification.json` records the run, source, test counts, startup checks and artifact metadata.

## Human qualification

Follow the [controller playtest guide](CONTROLLER_PLAYTEST.md) on the actual Mac and Ubuntu PC. Simulated input, native rendering and software-Vulkan startup cannot certify physical detection, USB/Bluetooth behavior, rumble, stick feel, sound or a complete human campaign run. Earlier candidates remain available for comparison.
