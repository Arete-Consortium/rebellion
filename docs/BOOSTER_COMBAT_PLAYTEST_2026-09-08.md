# Booster combat and campaign checkpoint — September 8

This candidate keeps the [approved classic booster artwork](CLASSIC_BOOSTER_PLAYTEST_2026-09-08.md) and connects the speed and damage bonuses to real combat. It also fixes a crash found during a scripted three-mission run.

Candidate: `dist/booster-combat-20260908/Rebellion.app`, with a ZIP beside it. Earlier candidates are preserved. This is a local Apple Silicon Mac playtest build.

## Gameplay fixes

- **Overclocker now changes actual movement by 1.5×.** Its timer previously drove the HUD and visuals without affecting movement. Thrust and the speed cap now use the temporary multiplier together, so friction cannot make the bonus imperceptible. Base hull and upgrade values remain stable.
- **Pyrolancea now doubles projectile damage.** Damage is multiplied when the player fires, preserving inventory and downstream weapon-family/Salt Miner modifiers. Refreshing a drug renews its duration; it does not multiply the bonus again. Already-fired shots retain their launch damage.
- **Ship overdrive no longer repeatedly multiplies the base speed.** Ability modifiers start neutral, resolve before player input systems, and combine once with the booster. Expiry restores ordinary movement. This covers speed abilities; it does not certify every other hull ability.
- **Renewed HUD bars recover their original color.** A booster collected during the low-time warning no longer leaves a renewed bar flashing red/orange.
- **Overlapping ship/scenery contacts no longer crash.** The Gallente probe reproduced a projectile being consumed by a ship before terrain resolution tried to despawn it again. Terrain now checks the live projectile and its remaining pierce charges, so a spent shot cannot damage scenery or spend an outdated charge. Ship-first contact priority is preserved.

No artwork, drug magnitudes, durations, drop probabilities, enemy health or difficulty values changed in this checkpoint. The development probes are examples and are not included in the app.

## Campaign run evidence

The final probe used the normal parallel executor and the actual **Play → chapter → faction → Newbro → starter hull → briefing** route. A scripted pilot supplied movement and aim input. Enemy damage/deaths, drops, player defenses, results and mission transitions used the production systems. No forced kills, invulnerability, health/stat edits or retries were used.

Both sides reached **SliceComplete**, with all three results screens and both boss defeats recorded:

| Route | Mission 1 | Mission 2 | Mission 3 | Active combat total |
| --- | ---: | ---: | ---: | ---: |
| Caldari / Kestrel | 64.3 s | 105.1 s | 134.7 s | 5 min 4 s |
| Gallente / Tristan | 87.0 s | 222.2 s | 144.7 s | 7 min 34 s |

Boss encounter times were **42.6 / 60.8 seconds** for Kestrel and **129.5 / 29.8 seconds** for Tristan. The reports contain successive declining health samples and the actual destruction events. These times include the pilot's accuracy and naturally acquired equipment/boosters; they are not controlled hull-DPS comparisons. Menu, briefing and boss-introduction time is excluded from the combat totals.

The pilot killed 108 of 120 observed enemies as Caldari and 87 of 120 as Gallente. Existing wave rules allow surviving ships to leave the screen; reaching the finale does not mean every ship was destroyed. It collected 3 of 24 spawned consumable drugs as Caldari and 4 of 15 as Gallente. These are useful pickup-routing observations, not a representative human collection rate.

Content/drop RNG uses unseeded thread-local randomness under normal scheduling. Results vary between runs. Earlier serial diagnostic runs and the original crash log are preserved separately in the evidence directory.

Reproduce the bounded probe with:

```sh
cargo run --offline --locked --example campaign_probe -- build/playtest-review/my-combat-probe
```

Each faction gets a fresh temporary save. The probe stops at victory, game over, or a 20-minute simulated-time limit and writes faction JSON reports. It is headless: it does not establish rendered performance, controller detection, human difficulty or sound quality.

## Validation

- **539 tests pass**, zero failures or ignored tests, across 37 target summaries. New regressions cover real booster movement/projectiles in Playing and BossFight, refresh/expiry, ability stacking, HUD refresh, and overlapping ship/terrain contacts with zero, one and two pierce charges.
- The booster regressions failed against the original runtime, and the collision regression reproduced the Gallente panic before its fix. Before/after logs are preserved.
- All-target Clippy with warnings denied, formatting and diff checks pass. The existing dependency notice for `block 0.1.6` remains.
- Native Metal review loads all 14 pickup textures, activates and renews real pickup effects, captures the refreshed cyan Overclocker bar, and verifies expiry. Its separate protected, stationary review pilot is an artwork/HUD fixture, not the campaign pilot. Existing B0003 menu-cleanup warnings remain; the successful review has no panic or missing assets.

Evidence: `build/playtest-review/booster-combat-20260908/`. `source-manifest.json` fingerprints the 327 source, test, example, asset, configuration and packaging inputs. `caldari.json` and `gallente.json` hold the final campaign measurements.

## Packaged Mac verification

The release package passed strict ad-hoc signature verification after clean ZIP extraction under `/private/tmp`. LaunchServices started that extracted app from an unrelated working directory with an isolated muted save. It initialized and stayed alive without a panic or missing-asset error; only the owned test instance was closed. The executable and bundled booster images match the candidate, and all 327 frozen inputs were unchanged through packaging.

- App: **57,806,210 bytes**, 136 files. ZIP: **32,118,179 bytes**.
- Executable SHA-256: `7b1561037b60074d510adfbdd736ffc8c3bef62f53d0aa008f2e0d9ab59a01ab`.
- ZIP SHA-256: `f7f8f6d67afc8e57fb1eebe9d358ac731209a26e09dd15e8cf8c0679728bf8c4`.
- `bundle-manifest.json`, `package.log` and the native launch logs record the checks. The campaign probe ran the matching source separately; this package smoke check establishes startup, not a full packaged human playthrough.

## Next work

Follow-up: the runtime ability gaps in item 2 are implemented and regression-tested in the [hull ability checkpoint](HULL_ABILITY_PLAYTEST_2026-09-08.md). The list below preserves this earlier candidate's handoff; human qualification and objective wording remain open.

1. Qualify the candidate with a human run: pickup readability while dodging, whether the Patrol Commander feels too slow in Tristan, the later power increase, sound levels, and physical controller behavior.
2. Audit the remaining advertised hull abilities through their actual consumers. Source review found range, damage-resistance and close-range damage modifiers being assigned in `ability.rs` without corresponding reads in the current firing/damage path. Follow-up review during Linux packaging also found that Afterburner's invulnerability flag is assigned without a damage-path consumer. Test and finish those effects before claiming each hull's full special behavior; the close-range ability needs an explicit distance rule. Exercise instant Salvo/Rocket Barrage activation while the ordinary weapon is cooling down, so paying the ability cost reliably produces the advertised burst.
3. Review objectives that say “destroy” while permitting wave escape, and tune pickup access using human observations. Avoid changing probabilities or boss health solely to optimize this pilot's route.

Public distribution, notarization and human/controller acceptance remain open. No commit, push or publication is part of this checkpoint.
