# Right-stick firing checkpoint — September 8

The owner clarified the intended feel: pushing the right stick aims and fires in that direction, while triggers and bumpers remain available for combat utilities. This replaces the RT-fire layout in the [previous controller-only candidate](CONTROLLER_ONLY_PLAYTEST_2026-09-08.md).

## Current layout

| Action | Xbox-style control |
| --- | --- |
| Move | Left stick, with analog strength |
| Aim and fire | Push right stick toward the target |
| Stop primary fire | Center the right stick |
| Thrust | RT |
| Hull ability / special burst | LT |
| Dodge | RB; left stick selects the side |
| Charged overload | LB |
| Previous / next ammunition | X / B or D-pad left / right; autocannon hulls |
| Interact / confirm | A |
| Menu navigation / back | Left stick or D-pad / B |
| Pause / resume | Menu |

Firing follows the weapon's normal cadence. Centered-stick noise stays inside the existing deadzone; the last aim direction is retained for hull special bursts. Holding a trigger or bumper does not repeatedly spend capacitor or reactivate an action. Holding an ammo button cycles once per press. Face-button ammo selection does not also navigate menus or steer the ship.

Boosters still activate on pickup. Manual booster storage is a separate follow-up; Y has no combat binding in the offered chapters. No booster inventory, drop tuning, art replacement or combat-stat change is included here.

The game remains controller-only. Connection acknowledgement, stable device selection, disconnect freezing, preserved pause state and protection from old keyboard mappings remain in place. The connection screen, controller guide, ammo label and overload label show the revised layout.

## Verification and candidates

**563 tests pass**, with no failures or ignored tests across 39 target summaries. All-target Clippy with warnings denied, formatting and diff checks pass. The controller integration suite now has twelve cases. It checks eight aim/fire directions and centered-stick noise in ordinary combat and boss fights; RT thrust without primary fire; separate LT/RB/LB actions; held-control suppression; old binding isolation; ammunition changes without movement; the full chapter/faction menu route; pause and disconnect protection.

The native review fixture captured the connection screen and controller guide at 800×700 logical resolution. Both were visually checked for correct labels and clipping. The fixture exited normally; existing B0003 menu-cleanup warnings and the `block 0.1.6` dependency notice remain. Synthetic input does not establish physical device support.

Both starter campaigns reached **SliceComplete** through actual controller menu navigation and ordinary right-stick firing, without forced kills, retries or combat-stat changes. Every mission completed and both boss health samples declined to destruction events.

| Route | Mission 1 | Mission 2 | Mission 3 | Active combat total |
| --- | ---: | ---: | ---: | ---: |
| Caldari / Kestrel | 52.3 s | 110.2 s | 94.0 s | 4 min 17 s |
| Gallente / Tristan | 77.0 s | 235.8 s | 287.4 s | 10 min 00 s |

Content/drop randomness remains unseeded. These timings are route observations, not a controlled balance comparison or human playtest. The longer Tristan encounters remain a human pacing check; this input correction does not tune boss health or pickups.

Source is on `codex/linux-playtest` at `4455c83bf522350c14b2bac4e4a0ffb59e0472e5`. All 336 frozen source, test, asset and packaging inputs match that Git tree.

**Apple Silicon Mac:** `dist/twin-stick-20260908/Rebellion.app`, with `Rebellion.app.zip` beside it. A fresh ZIP extraction under `/private/tmp` passed strict signature verification and a normal LaunchServices startup from an unrelated working directory. The test used an isolated muted save, initialized successfully and stayed alive for twelve seconds without a panic or missing assets. The owned test instance was then closed.

- App: 57,789,602 bytes across 136 files; ZIP: 32,119,550 bytes.
- Executable SHA-256: `22c4014c69a17eb7579c80a1371a5248fac2dea717bf43d1b429ec2c8d23a2e6`.
- ZIP SHA-256: `be4c0e6165812897b243462e62ea582811b50adb958cf4f4445d706b4689c07c`.
- Extracted executable/assets match the candidate; all 336 frozen inputs remained unchanged. `bundle-manifest.json`, `package.log` and `packaged-startup.log` record the checks.

**Ubuntu / Linux Intel/AMD x86_64:** [download the tested package](https://github.com/Arete-Consortium/rebellion/actions/runs/34304704804/artifacts/10086401586) — 35,873,256 bytes (about 36 MB). GitHub sign-in is required. This Actions artifact expires October 9, 2026 UTC. Follow the [Linux instructions](LINUX_PLAYTEST.md) to extract and run it.

[Ubuntu 22.04 qualification](https://github.com/Arete-Consortium/rebellion/actions/runs/34304704804) passed all **563 tests**, with no failures or ignored tests across 39 targets, then built the optimized release. A fresh extraction passed checksums and shared-library resolution, initialized from an unrelated working directory with an isolated muted save, and stayed alive through the startup check without a panic or missing assets. CI used Xvfb and software Vulkan. The [startup evidence](https://github.com/Arete-Consortium/rebellion/actions/runs/34304704804/artifacts/10086401953) contains its log and machine-readable result.

The Linux artifact records source `4455c83bf522350c14b2bac4e4a0ffb59e0472e5`, matching the Mac candidate's 336 frozen inputs. GitHub artifact digest: `sha256:1c1649fb36b8570e27bd90fe849a6baeff7f24c57ffe34ba622d28f334b69eeb`. The included tarball checksum is separate. `linux-ci-verification.json` records the source, test counts, startup checks and artifact metadata. Earlier candidates are preserved for comparison.

Evidence is kept under `build/playtest-review/twin-stick-20260908/`, including the previous files, focused diff, test logs and frozen input manifest. The campaign pilot now fires solely through the right stick in controller mode; it no longer holds RT, which would activate thrust under the revised layout.

Physical controller detection, stick feel, sound, rumble and complete human playthroughs remain separate checks. Use the [controller guide](CONTROLLER_PLAYTEST.md), including centering the stick to stop fire, utility use while aiming, and reconnecting with controls held.
