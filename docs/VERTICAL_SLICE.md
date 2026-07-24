# Rebellion — Vertical Slice Definition

**Version**: v1.0-vertical-slice
**Date**: 2026-07-24
**Scope**: Caldari-Gallente missions 1–3 only

---

## What Is This?

A self-contained 20–30 minute gameplay experience demonstrating the core loop of Rebellion: faction selection, ship loadout, three missions of escalating intensity, boss encounters, and meaningful progression.

**Not included in this slice**: Missions 4–5, Elder Fleet, Triglavian Invasion, Abyssal Depths, Shiigeru Nightmare endless mode, full upgrade tree, achievements beyond the first 3 missions.

---

## Missions

### Mission 1: Orbital Skirmish
- **Duration**: ~5 minutes
- **Enemies**: 3 waves of frigate patrols
- **Tutorial elements**: Movement, firing, heat system introduction
- **Objective**: Destroy enemy patrol ships
- **Unlocks**: Nothing (starter mission)

### Mission 2: Urban Firefight
- **Duration**: ~7 minutes
- **Enemies**: 4 waves + Patrol Commander mini-boss
- **Objective**: Clear the airspace above Caldari Prime cities
- **Boss**: Patrol Commander (2 phases, 400 HP)
- **Unlocks**: Mission 3 access

### Mission 3: Fleet Interdiction
- **Duration**: ~10 minutes
- **Enemies**: 5 waves + Fleet Commander boss
- **Objective**: Intercept enemy reinforcements before they reach the front
- **Boss**: Fleet Commander (3 phases, 700 HP)
- **Unlocks**: Ship upgrade slot (first meaningful choice)

**End of slice**: After defeating the Fleet Commander, the player sees a "Vertical Slice Complete — Full Campaign Coming Soon" screen with a summary of their performance.

---

## Ship Progression

### Available at Start (Mission 1)
| Faction | Ship | Role |
|---|---|---|
| Caldari | Hawk | Missile Boat (starter) |
| Caldari | Harpy | Railgun Platform |
| Gallente | Enyo | Blaster Brawler (starter) |
| Gallente | Ishkur | Drone Boat |

### Unlocked During Slice
| Mission | Unlock |
|---|---|
| Mission 1 complete | — |
| Mission 2 complete | Shield Hardening upgrade (choice) |
| Mission 3 complete | Weapon Overclock upgrade (choice) |

**No T3 destroyers in slice.** Jackdaw/Hecate remain locked.

---

## Enemy Roster (Slice)

| Type | Behavior | Threat |
|---|---|---|
| Condor/Kestrel/Merlin (Caldari enemies) | Linear, Zigzag | Low |
| Atron/Incursus/Tristan (Gallente enemies) | Homing, Weaver | Low–Medium |
| Patrol Commander (Mission 2 boss) | Sweep + spread shot | Medium |
| Fleet Commander (Mission 3 boss) | Phase transitions + enrage | High |

**No elite enemies in slice.** Standard frigates + 2 bosses only.

---

## Save Behavior

- Progress is saved after each mission completion
- On restart, player can continue from any completed mission (1, 2, or 3)
- High score and best chain are persisted per faction pair
- Ship unlocks and upgrades are NOT persisted across slice boundaries (slice is self-contained)

---

## Entry Points

### From Main Menu
- **"PLAY VERTICAL SLICE"** — jumps directly to faction select, then missions 1→2→3
- **"PLAY FULL CAMPAIGN"** (if enabled in non-slice builds) — continues past mission 3

### For Playtests / Demos
- `--vertical-slice` CLI flag forces slice mode regardless of save state
- WASM build defaults to slice mode for browser demo

---

## Success Criteria

A human playtester should be able to:
1. Start from main menu and reach Mission 1 within 30 seconds
2. Understand movement and firing by end of Mission 1 without reading a manual
3. Feel a clear difficulty escalation from Mission 1 → 2 → 3
4. Recognize the Patrol Commander as a threat before it fires
5. Survive Mission 3 on "Newbro" difficulty on their second or third attempt
6. See a coherent "Vertical Slice Complete" screen with meaningful stats
7. Complete the full slice in under 35 minutes

---

## Known Limitations (Acceptable for Slice)

- Art is placeholder sprites
- Audio is procedural/generated
- Only Caldari vs Gallente faction pair is playable
- No difficulty above "Newbro" in slice
- No leaderboard integration
- No controller haptics
- No Steam Deck-specific UI

---

## Post-Slice Roadmap

| Phase | Content |
|---|---|
| v1.1 | Missions 4–5 + T3 destroyers |
| v1.2 | Elder Fleet campaign (13 missions) |
| v1.3 | Triglavian Invasion + Abyssal Depths |
| v1.4 | Full upgrade tree + achievements |
| v1.5 | Steam Deck + controller polish |
| v2.0 | Network multiplayer (stretch) |

**This document is append-only.** Changes to the slice scope require an ADL entry.
