# Booster pickup and HUD pass — September 8

This pass replaces the relevant consumable pickup artwork with six verified EVE booster icons. It follows the [transport objective candidate](TRANSPORT_PLAYTEST_2026-09-08.md).

Open **`dist/boosters-20260908/Rebellion.app`** for the updated Mac playtest. The clean archive is beside it at `Rebellion.app.zip`; the previous transport build is preserved.

| Pickup effect | Artwork |
| --- | --- |
| Shield +25 | Standard Blue Pill |
| Armor +25 | Standard Exile |
| Capacitor +50 | Standard Mindflood |
| Speed boost, 5 seconds | Agency Overclocker |
| Double damage, 10 seconds | Agency Pyrolancea |
| Full shield, armor and hull repair | Agency Hardshell |

These are Rebellion's arcade effects and durations. Hardshell is a thematic adaptation of the shield/armor repair booster. Hull repair, invulnerability, nanites and weapon modifications keep their equipment artwork. The full-repair pickup previously used an unrelated speed-booster image.

The active-effect HUD now uses the same cached images as the world pickups. It shows the item name and actual game effect above the countdown bar. Textured pickups remain upright while their rarity pulse and orbital particles animate. The capacitor icon now uses the same readable base size as the other booster pickups.

All six files are unmodified 64×64 RGBA PNGs fetched from CCP's image service, totaling **39,093 bytes**. They are bundled for offline play and loaded through the existing Bevy AssetServer; no runtime network access or new dependency was added. [Artwork sources, type IDs and checksums](../assets/powerups/README.md) are included beside the assets.

## Verification

- All **16 existing collectible unit tests pass**. All-target Clippy with warnings denied, formatting and diff checks pass. The existing `block 0.1.6` future-Rust compatibility notice remains.
- The native renderer loaded all **14 powerup textures**. The six new icons were checked for 64×64 dimensions and transparent/opaque pixels. The captured pickups are upright, and the HUD shows the same artwork with legible item/effect labels.
- Real pickup-effect events activated Overdrive, Damage Boost and Invulnerability. Their timers expired normally and the three HUD panels disappeared in the second capture.
- The native run completed without a panic or missing asset. Existing B0003 menu-cleanup warnings remain, as recorded in the transport checkpoint.

The native review tool is `cargo run --offline --locked --example powerup_review`. It uses an isolated muted save and a stationary pilot with extra hull for the timer observation, the real pickup spawn path and pickup-effect events. It captures active and expired HUD states, then exits. The labeled pickup arrangement is a review fixture, not an added game screen. Screenshots, check logs and source/asset digests are under `build/playtest-review/boosters-20260908/`.

This focused presentation pass does not repeat the prior full 533-test suite or establish a human campaign/controller playtest. The existing transport and campaign acceptance work remains open.

## Packaged candidate

The release build passed packaging and strict ad-hoc signature verification after clean ZIP extraction under `/private/tmp`. LaunchServices started that extracted app with its own muted save. It initialized and remained alive without a panic or missing-asset error, then only the test instance was closed. Its executable and bundled booster artwork match the source candidate.

- App: **57,800,189 bytes**, 133 files. ZIP: **32,111,133 bytes**.
- Executable SHA-256: `d78c12e07c7f1c62db5f8ce1e7c8465b57db5a47472febdef35899daec0a67ad`.
- ZIP SHA-256: `79b79c2756bf61cfa853c262683932ffdbb61e167488ac9d40ecf615e204a790`.
- Package/launch results: `build/playtest-review/boosters-20260908/bundle-manifest.json`. The source manifest records 281 source and asset digests; they remained unchanged through packaging and verification.

No commit, push, publication or notarization was performed.
