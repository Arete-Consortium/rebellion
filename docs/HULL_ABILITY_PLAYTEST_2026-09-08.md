# Hull ability combat checkpoint — September 8

This checkpoint connects the advertised hull effects to actual combat. It follows the [booster combat checkpoint](BOOSTER_COMBAT_PLAYTEST_2026-09-08.md). Source is on the `codex/linux-playtest` testing branch at `edef0f6c44e6dcc781f1e35f321bb90e6863593c`.

## Changes and explicit rules

| Ability | Runtime behavior |
| --- | --- |
| Armor Hardener | Halves incoming damage before ordinary damage-type resistance and shield/armor/hull routing. Covers enemy projectiles, scenery contacts, disintegrator beams and Abyssal hazards. |
| Afterburner | Its active invulnerability now protects against the same damage sources. Existing booster, maneuver and spawn immunity also protect against Abyssal hazards. |
| Rocket Barrage / Salvo | Activation queues one three-/four-shot volley, including while the ordinary weapon is cooling down and without holding fire. Inventory extra shots still stack. The burst preserves the ordinary cooldown. |
| Scorch | Laser range is 500 world units normally and 750 while active. Each shot retains the range captured at launch. |
| Close Range | Doubles direct projectile damage at impacts within 200 world units of the player's position when firing, including the boundary. Chain lightning checks each secondary impact independently. Moving the player or ending the ability does not change a fired shot. |
| Warp Disruptor | Enemies within 300 world units of the player move at 50% speed. Covers ordinary enemies and both boss implementations. Attack clocks retain their usual rate; leaving the field or expiry restores movement without changing base speed. |

The previous laser lifetime allowed roughly 1,200 units of travel, beyond the battlefield. Defining a meaningful Scorch bonus therefore **reduces ordinary laser range to 500**. This is an intentional balance choice that needs human range/feel testing on Amarr hulls. Other weapon baseline ranges are unchanged.

Ability expiry resolves before current-tick combat consumers. Regression checks also exercise Shield Boost, Armor Repair, drone spawning/damage/expiry, activation cost and pause behavior. This pass does not certify every hull's balance or input binding.

No artwork, drug drop probabilities, enemy health or boss health changed. Classic finished-drug artwork remains intact.

## Validation

- **551 tests pass**, zero failures or ignored tests across 38 target summaries. The new integration suite covers live firing/damage paths, cooldown bursts, exact distance boundaries, protection, drones, healing, boss movement and expiry.
- The initial defensive, range, close-damage and burst regressions failed against the previous runtime; a separate regression reproduced Warp Disruptor's missing slowdown before repair.
- All-target Clippy with warnings denied, formatting and diff checks pass. The existing `block 0.1.6` future-compatibility notice remains.
- All **332 frozen source, asset, test and packaging inputs** match the GitHub build tree. All assets match the earlier booster checkpoint.

Evidence directory: `build/playtest-review/hull-abilities-20260908/`. It contains before/after tests, the complete test and Clippy logs, the focused diff, source hashes and campaign JSON reports.

## Complete campaign probe

The existing bounded headless pilot followed Play → chapter → faction → Newbro → starter hull → briefing, with ordinary movement and aim input, normal parallel scheduling, no forced kills, no altered combat stats, no retries and fresh isolated saves. Both sides reached **SliceComplete**, passing all three results screens and defeating both bosses. Successive health samples decrease in each boss encounter.

| Route | Mission 1 | Mission 2 | Mission 3 | Active combat total |
| --- | ---: | ---: | ---: | ---: |
| Caldari / Kestrel | 61.2 s | 112.6 s | 131.5 s | 5 min 5 s |
| Gallente / Tristan | 89.2 s | 232.4 s | 292.4 s | 10 min 14 s |

Boss encounters lasted 42.7 / 60.8 seconds for Kestrel and 141.7 / 182.7 seconds for Tristan. These runs have unseeded content/drop randomness and different acquired equipment; they are not controlled DPS comparisons. The longer Tristan encounters are a pacing observation for human review, not a reason to tune boss health to this pilot. Existing wave escape rules remain; the pilot recorded 108/120 kills as Caldari and 86/120 as Gallente.

Reproduce with:

```sh
cargo run --offline --locked --example campaign_probe -- build/playtest-review/my-ability-probe
```

The integration tests exercise the abilities directly. The campaign probe establishes route continuity under ordinary combat; it does not measure rendered performance, sound quality, physical controller behavior or human difficulty.

## Candidate qualification

**Apple Silicon Mac:** `dist/hull-abilities-20260908/Rebellion.app`, with `Rebellion.app.zip` beside it. Release packaging passed strict ad-hoc signature verification after a clean ZIP extraction under `/private/tmp`. LaunchServices started the extracted app from an unrelated working directory with an isolated muted save. It initialized and stayed alive for more than 30 seconds without a panic or missing assets; only that owned test instance was closed. The first sandboxed launch did not initialize; the normal approved LaunchServices launch succeeded.

- App: 57,822,658 bytes, 136 files. ZIP: 32,124,892 bytes.
- Executable SHA-256: `ecdfe23a9da30af102feaa425097ba858b8ec311a5ec9ed84ce6ad1252f5fdb2`.
- ZIP SHA-256: `9cc1f2b154456bac6669d0b073e51912ad7db8b9c9054be5e6d1bec2bce0b3e4`.
- Extracted executable and all bundled assets match the candidate; all 332 frozen inputs remained unchanged through packaging. `bundle-manifest.json`, `package.log` and `launchservices-startup.log` record the checks.

**Ubuntu / Linux Intel/AMD x86_64:** [download the tested package](https://github.com/Arete-Consortium/rebellion/actions/runs/34300065734/artifacts/10084777046) — 35,881,438 bytes (about 36 MB). GitHub sign-in is required. This Actions artifact expires October 9, 2026 UTC. Use the [Linux instructions](LINUX_PLAYTEST.md) to extract and run it.

[Linux build 34300065734](https://github.com/Arete-Consortium/rebellion/actions/runs/34300065734) passed on Ubuntu 22.04 with Rust 1.98.1: **551 tests**, zero failures or ignored tests, then an optimized release build. A fresh extraction passed checksums and shared-library resolution, initialized from an unrelated directory with an isolated muted save, and stayed alive for at least ten seconds without a panic or missing assets. [Startup evidence](https://github.com/Arete-Consortium/rebellion/actions/runs/34300065734/artifacts/10084777283) contains its log and machine-readable result; CI used Xvfb and software Vulkan.

The Linux artifact is built from `edef0f6c44e6dcc781f1e35f321bb90e6863593c`, matching the Mac candidate's 332 frozen inputs. GitHub artifact digest: `sha256:fd33fdd5c7c5de435029c9c02d4eea3f7bca01b1ddb8eeaf2feb93249408178d`. The included tarball checksum is separate. Earlier candidates remain available. Startup checks do not establish hardware performance, controller/audio acceptance or a complete human playthrough.

## Next work

1. **RBL-05: controls.** Shift still overlaps thrust and hull ability activation. The ability consumer still uses hardcoded Shift/right-trigger input instead of the remappable ability action; right trigger also fires and can reactivate an ability when its cooldown finishes. Define separate actions, consume the configured binding, and test remap/reset/reload/pause before physical-controller acceptance. Ability cost tests isolate the ability from the separate thrust cost.
2. **Human playtest when the owner is available.** Check Amarr's new baseline range, close-range positioning, slow-field readability, burst feedback, Tristan boss pacing, booster pickup readability and sound levels. Test the actual connected controller on each intended platform.
3. **Truthful objectives (RBL-01).** Review objectives that say “destroy” while permitting wave escape. Keep pickup/boss tuning tied to observed human play. RBL-06 separately covers retained run statistics and best-score comparison.

Public distribution, notarization and complete human/controller acceptance remain open. This is a testing-branch candidate, not a release approval.
