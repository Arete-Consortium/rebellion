# Booster pickup artwork

The main consumable set uses the classic finished-drug vial icons from the [EVE University medical booster reference](https://wiki.eveuniversity.org/Medical_boosters): Blue Pill, Exile, Mindflood and X-Instinct. Newer finished boosters cover the remaining consumable effects. These are unmodified 64×64 RGBA PNGs, checked visually on 2026-09-08.

EVE Online artwork belongs to CCP Games and is separate from this repository's MIT-licensed code. Rebellion is an unofficial free fan project. Archived classic images retain their original pixels; no generated, recolored or manufacturing-material art is substituted.

| Family | File | Consumable type ID | Rebellion effect | Artwork source |
| --- | --- | --- | --- | --- |
| Blue Pill | `blue_pill_booster.png` | [9950](https://esi.evetech.net/latest/universe/types/9950/?language=en), group 303 | Shield +25 | [PNG](https://wiki.eveuniversity.org/images/1/1f/Icon_blue_pill.png) |
| Exile | `exile_booster.png` | [15479](https://esi.evetech.net/latest/universe/types/15479/?language=en), group 303 | Armor +25 | [PNG](https://wiki.eveuniversity.org/images/3/3c/Icon_exile.png) |
| Mindflood | `mindflood_booster.png` | [15463](https://esi.evetech.net/latest/universe/types/15463/?language=en), group 303 | Capacitor +50 | [PNG](https://wiki.eveuniversity.org/images/d/d6/Icon_mindflood.png) |
| X-Instinct | `x_instinct_booster.png` | [15457](https://esi.evetech.net/latest/universe/types/15457/?language=en), group 303 | Invulnerability for 3 seconds | [PNG](https://wiki.eveuniversity.org/images/1/1b/Icon_x-instinct.png) |
| Overclocker II | `overclocker_booster.png` | [46005](https://esi.evetech.net/latest/universe/types/46005/?language=en), group 303 | Speed boost for 5 seconds | [PNG](https://images.evetech.net/types/46005/icon?size=64) |
| Pyrolancea II | `pyrolancea_booster.png` | [45999](https://esi.evetech.net/latest/universe/types/45999/?language=en), group 303 | Double damage for 10 seconds | [PNG](https://images.evetech.net/types/45999/icon?size=64) |
| Hardshell I | `hardshell_dose_i_booster.png` | [46001](https://esi.evetech.net/latest/universe/types/46001/?language=en), group 303 | Hull +25 | [PNG](https://images.evetech.net/types/46001/icon?size=64) |
| Hardshell II | `hardshell_booster.png` | [46002](https://esi.evetech.net/latest/universe/types/46002/?language=en), group 303 | Full shield, armor and hull repair | [PNG](https://images.evetech.net/types/46002/icon?size=64) |
| Wightstorm Sunyata I | `sunyata_booster.png` | [90690](https://esi.evetech.net/latest/universe/types/90690/?language=en), group 303 | Heat -50 | [PNG](https://images.evetech.net/types/90690/icon?size=64) |

## Mapping and import rules

- The classic references identify the drug family, without EVE dose-tier badges. The listed ESI IDs verify finished consumable identities (group 303, Booster). They are not download instructions for the classic artwork.
- **Do not blindly replace the classic vials with current type-icon downloads.** The image service returns identical bytes for Standard Blue Pill (9950, group 303) and Pure Standard Blue Pill (25237, group 712, Biochemical Material). That shared pile/container artwork was rejected for this game. Both comparison files and type responses are preserved in `build/playtest-review/classic-boosters-20260908/`.
- X-Instinct's defensive theme represents Rebellion's existing three-second invulnerability. Hardshell's repair theme represents hull-only and full-repair pickups. Sunyata's heat-management theme represents instant heat reduction. These are explicit arcade adaptations; they do not reproduce EVE's exact stat mechanics, booster slots, side effects or dose percentages.
- Sunyata's verified source item is named `Expired Wightstorm Sunyata Booster I` in current ESI. This imports the original finished-drug icon; EVE's calendar expiration is not a Rebellion mechanic.
- Overclocker and Pyrolancea cover speed and direct damage because the classic set has no matching movement-speed or flat-damage drug. Drop, Crash, Frentix and Sooth Sayer are reserved for future tracking, missile application or range effects; their icons are not shipped as unused assets.
- All nine consumable pickups now use booster artwork. Persistent weapon modifications keep their weapon/module icons. No gameplay effect magnitude or duration changed. The normal drop pool now includes Mindflood's previously unreachable capacitor pickup.

## SHA-256

- `blue_pill_booster.png`: `8bd5e35fc8b3cdeb18f1a57205cfb2d3304a013ccc2c04cf4592830d6efa4d05`
- `exile_booster.png`: `cb772348959c93f7ae2bb5ffa10842d3ab7a28d41c907d67bf1845163922300c`
- `mindflood_booster.png`: `1e1bf6e046c7e04a709fcedf2bfa6d2f93012ed970e787a231a642bd5c6ee7b2`
- `x_instinct_booster.png`: `578d64f3d10c564d6d09a1a30e55c85db4d7707a0388131b16269286b823280d`
- `overclocker_booster.png`: `bddfc12e50b85600a0791a0ceee0f1fdec955ffe1380e8043a2cd27e6a584391`
- `pyrolancea_booster.png`: `dcc82092899d2630a6366c1f0e5e1b5dc25796a7dc3bc5cf1e141ec13cb8d08e`
- `hardshell_dose_i_booster.png`: `a57ec7780e28500e61cc807f7f455eb3fdefedb810438a3cdfd8a6809fbdcc1b`
- `hardshell_booster.png`: `ae51d1bb7da26807bb0c7ff618088de72de58b62e194e54bbb98077929cd1f3a`
- `sunyata_booster.png`: `3c2624b1660d16159ba625f07974cda4522efd05a57d6d783554c5de81a8c233`

Total active booster artwork: **41,894 bytes** across nine files.
