# Classic booster correction — September 8

This supersedes the artwork in the [first booster candidate](BOOSTER_PLAYTEST_2026-09-08.md). The core drugs now use their classic finished-drug bottle images: Blue Pill, Exile, Mindflood and X-Instinct. Newer boosters cover the remaining consumable roles.

Open **`dist/classic-boosters-20260908/Rebellion.app`**. The ZIP beside it contains the same candidate. Earlier builds are preserved.

The previous import checked finished item IDs but still selected unsuitable artwork. Current EVE Image Server responses for Standard Blue Pill (9950, Booster group 303) and Pure Standard Blue Pill (25237, Biochemical Material group 712) are byte-identical. Correct type IDs alone do not establish the intended image. The classic bottles now come from the archived drug-family images in the EVE University medical booster reference. All selected sources, identity checks and file hashes are recorded in [the asset notes](../assets/powerups/README.md).

| Pickup | Finished-drug artwork | Rebellion effect |
| --- | --- | --- |
| Shield | Classic Blue Pill | Restore 25 shield |
| Armor | Classic Exile | Restore 25 armor |
| Capacitor | Classic Mindflood | Restore 50 capacitor |
| Defense | Classic X-Instinct | Three seconds of invulnerability |
| Speed | Agency Overclocker II | Five seconds of overdrive |
| Damage | Agency Pyrolancea II | Ten seconds of double damage |
| Hull repair | Agency Hardshell I | Restore 25 hull |
| Full repair | Agency Hardshell II | Restore shield, armor and hull |
| Cooling | Wightstorm Sunyata I | Remove 50 heat |

X-Instinct, Hardshell and Sunyata use clear arcade adaptations of their defensive, repair and heat-management themes. EVE's exact mechanics and item expiration dates are not simulated. Persistent weapon modifications remain equipment pickups. The four other classic families (Drop, Crash, Frentix and Sooth Sayer) remain candidates for future matching tracking/application/range effects, without adding unused artwork or new mechanics here.

All nine images are unmodified 64×64 RGBA PNGs, totaling **41,894 bytes**. They share the existing offline asset cache with the HUD. The three timed indicators show Overclocker, Pyrolancea and X-Instinct alongside the actual game effect.

## Drop correction

Mindflood's capacitor effect and icon existed but no normal drop branch could choose it. The normal drop pool now assigns it 6%, reducing Overclocker from 10% to 8%, Pyrolancea from 8% to 6%, and cooling from 6% to 4%. Credits (20%), smart health drops (30%), defense (4%), full repair (2%) and weapon modifications (20%) retain their existing shares. Effect magnitudes and durations are unchanged. These are initial distribution values, subject to human playtest tuning.

A seeded regression exercises the real drop-spawn function and checks that every consumable can appear, including the previously unreachable capacitor pickup.

## Verification

- **534 tests pass**, none failed or ignored. This includes the new drop reachability regression and the existing campaign, transport, save, controls and replay suites.
- The first full run exposed an unrelated flaky test that required random Y separation in wave one's horizontal formation. It now checks the guaranteed horizontal spread; no formation gameplay changed. The complete suite then passed.
- All-target Clippy with warnings denied, formatting and diff checks pass. The existing `block 0.1.6` future-Rust compatibility notice remains.
- Native rendering loaded all 14 powerup textures and verified dimensions and alpha for the nine selected drugs. The screenshot shows the classic bottles without manufacturing piles or containers, with matching X-Instinct HUD artwork. Actual pickup-effect events activated all three timed buffs, and all three HUD panels disappeared when their timers expired.
- The native review completed without a panic or missing asset. Existing B0003 menu-cleanup warnings remain. The fixture uses its own muted save and a protected stationary pilot; this is separate from human difficulty or controller qualification.

Evidence is stored in `build/playtest-review/classic-boosters-20260908/`; the native review is reproduced with `cargo run --offline --locked --example powerup_review`. The labeled grid is a review fixture, not a game screen.

## Packaged Mac verification

The release build passed packaging and strict ad-hoc signature verification after clean ZIP extraction under `/private/tmp`. LaunchServices started that extracted copy with an isolated muted save; it initialized and remained alive without a panic or missing-asset error. Its executable and booster images match the candidate. Only the test instance was closed.

- App: **57,806,050 bytes**, 136 files. ZIP: **32,115,862 bytes**.
- Executable SHA-256: `929189df08405a93fdc763e6a445e90c17a383af748148ad036f009ee6c008dd`.
- ZIP SHA-256: `d3a16443385595d72418836bbdd9e297198125c5f308b96dbd15173351862114`.
- `bundle-manifest.json` records package and launch checks; `source-manifest.json` records 318 source, test and asset inputs, unchanged through packaging.

No commit, push, publication or notarization was performed. Human campaign, difficulty and physical-controller acceptance remain separate gates.
