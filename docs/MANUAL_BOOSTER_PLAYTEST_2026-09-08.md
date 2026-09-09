# On-demand booster checkpoint — September 8

The owner selected on-demand booster use for more tactical choice. This follows the [right-stick firing candidate](TWIN_STICK_PLAYTEST_2026-09-08.md).

## Player behavior

| Stored drug | Effect after pressing Y | Capacity |
| --- | --- | --- |
| Overclocker | Speed x1.5 for 5 seconds | 3 doses |
| Pyrolancea | Damage x2 for 10 seconds | 3 doses |
| X-Instinct | Invulnerability for 3 seconds | 3 doses |

- Collecting a timed drug stores one dose. **D-pad up/down** selects an occupied slot, wrapping past empty slots; **Y** uses the selected dose.
- An empty selection or an already-active effect spends nothing. Holding Y through expiration cannot automatically use another dose. Different effects can coexist.
- Full slots leave excess pickups in space until capacity is available or the pickup expires. Same-frame overlapping pickups cannot overfill a slot.
- Unused doses carry between missions and through boss transitions and pause. Death, a new run/hull selection or explicit restart clears them. Doses are run-local and are not written to the save file.
- Repairs, capacitor, cooling and full-repair pickups remain immediate. Weapon modules and their stacking behavior are unchanged.
- Right-stick aim/fire, RT thrust, LT hull ability, RB dodge, LB overload and X/B or D-pad left/right ammo remain separate. Booster selection does not steer or change ammo; Y does not trigger overload.

The HUD uses the same approved drug textures as world pickups, with a selected-slot outline, per-drug dose counts, effect description and a Y action or active countdown. Existing active-effect indicators remain visible. Pickup audio signals storage; the specific drug sound and rumble play on successful activation. No new art, drop probabilities, boss health or weapon statistics are introduced.

## Verification and candidates

**570 tests pass**, with no failures or ignored tests across 39 target summaries. Formatting and all-target Clippy with warnings denied pass. The controller integration suite now has 19 cases, including storage, same-frame capacity limits, selection isolation, held-Y protection, pause/restart, mission carryover and disconnect/reconnect. Combat tests verify the actual speed and projectile-damage multipliers, ability composition, and X-Instinct blocking real enemy projectile damage until expiration.

The native review loaded all 14 collectible icons and exercised the real pickup spawn path. Stored, active and expired-with-a-reserve-dose HUD captures were visually checked at 800×700 logical resolution. Dose counts, selection, countdowns and the approved drug artwork are readable; the twelve-row controller guide fits. The fixture exited normally. Existing B0003 menu-cleanup warnings, synthetic-controller rumble warnings and the `block 0.1.6` dependency notice remain. These muted scripted reviews do not qualify physical rumble or sound levels.

Both starter campaigns reached **SliceComplete** through controller menu navigation and ordinary right-stick firing, without retries, forced kills or combat-stat changes. All three missions completed for each faction; both bosses on each route showed declining health and emitted destruction events. The pilot used stored Pyrolancea and Overclocker in Caldari mission 1, and Pyrolancea in Gallente mission 3.

| Route | Mission 1 | Mission 2 | Mission 3 | Active combat total |
| --- | ---: | ---: | ---: | ---: |
| Caldari / Kestrel | 60.4 s | 109.9 s | 135.3 s | 5 min 06 s |
| Gallente / Tristan | 73.5 s | 238.7 s | 279.9 s | 9 min 52 s |

These are scripted observations with unseeded content/drop randomness, not a controlled balance comparison. The longer Tristan boss encounters remain a human pacing check. No probabilities or boss health were changed to favor the pilot.

Source is on `codex/linux-playtest` at `80f311f4d51ca6d2b283bb15c8370c3241fad3e6`. All 338 frozen source, test, asset and packaging inputs match that Git tree.

**Apple Silicon Mac:** `dist/manual-boosters-20260908/Rebellion.app`, with `Rebellion.app.zip` beside it. A fresh ZIP extraction under `/private/tmp` passed strict signature verification and a normal LaunchServices startup from an unrelated working directory. The test used an isolated muted save, initialized successfully and stayed alive for twelve seconds without a panic or missing assets. The owned test instance was then closed.

- App: 57,855,506 bytes across 136 files; ZIP: 32,133,688 bytes.
- Executable SHA-256: `02e44d23fef7e5d2b28270e5ac393515e0a91c72c6b321eec9d3d979f2b80122`.
- ZIP SHA-256: `ad3e2414a378aee7215533c6167bb782040583fbe6d8da65a73ffd8f22a99937`.
- Extracted executable/assets match the candidate; all 338 frozen inputs remained unchanged. `bundle-manifest.json`, `package.log` and `packaged-startup.log` record the checks.

**Ubuntu / Linux Intel/AMD x86_64:** [download the tested package](https://github.com/Arete-Consortium/rebellion/actions/runs/34306848068/artifacts/10087123527) — 35,886,866 bytes (about 36 MB). GitHub sign-in is required. This Actions artifact expires October 9, 2026 UTC. Follow the [Linux instructions](LINUX_PLAYTEST.md) to extract and run it.

[Ubuntu 22.04 qualification](https://github.com/Arete-Consortium/rebellion/actions/runs/34306848068) passed all **570 tests**, with no failures or ignored tests across 39 targets, then built the optimized release. A fresh extraction passed checksums and shared-library resolution, initialized from an unrelated working directory with an isolated muted save, and stayed alive through the startup check without a panic or missing assets. CI used Xvfb and software Vulkan. The [startup evidence](https://github.com/Arete-Consortium/rebellion/actions/runs/34306848068/artifacts/10087123808) contains its log and machine-readable result.

The Linux artifact records source `80f311f4d51ca6d2b283bb15c8370c3241fad3e6`, matching the Mac candidate's 338 frozen inputs. GitHub artifact digest: `sha256:696439edc4d725ea838b4fc0a2b95f807d68920abeb06e12652ee54a467f8042`. The included tarball checksum is separate. `linux-ci-verification.json` records the source, test counts, startup checks and artifact metadata.

Evidence directory: `build/playtest-review/manual-boosters-20260908/`. It preserves the previous files, focused diff, test logs, native captures, frozen source manifest and package evidence. Earlier candidates remain available for comparison.

Physical controller detection, stick feel, activation readability during dodging, audio levels and complete human playthroughs still need the [controller playtest](CONTROLLER_PLAYTEST.md). Automated checks and scripted pilots do not establish those outcomes.
