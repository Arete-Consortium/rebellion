# Rebellion — Archive Narrative

**Version**: 1.0
**Date**: 2026-07-25
**Status**: Active — pending integration into menu systems and opening sequence

---

## The Fiction

The year is YC 127.

CONCORD has completed a decades-long project: the **Capsuleer Combat Archive** — a neural-simulation program that reconstructs the defining conflicts of New Eden using authenticated battlefield telemetry, fleet records, and historical archives.

You are a capsuleer accessing this archive.

Your objective is not to rewrite history.
Your objective is to survive it.

---

## Opening Sequence

**INT. BLACK SCREEN**

*Silence. Then —*

> *Fluid draining. A hiss of pressurized gas.*

A single line of text appears, green on black:

```
YC 127
```

A second line:

```
CONCORD CAPSULEER COMBAT ARCHIVE
```

Third:

```
HISTORICAL RECONSTRUCTION PROGRAM
VERSION 4.2
```

A voice — calm, synthetic, female:

> **ARCHIVE:** "Access granted."

The screen flickers. Neural interface telemetry streams across the display — meaningless to the player, but atmospheric:

```
> Neural sync: ONLINE
> Clone pod: PRIMED
> Fleet telemetry: STREAMING
> Archive integrity: 99.91%
```

A technician's voice, distant, slightly distorted:

> **TECH:** "Simulation loaded. Historical accuracy: 99.84%."

The synthetic voice returns:

> **ARCHIVE:** "Capsuleer... prepare to relive history."

**FADE TO:**

Directly into gameplay. No main menu yet — the player is already in space, in the opening mission.

*(The main menu appears after Mission 1 is complete or the player pauses.)*

---

## Main Menu Structure

The menu is the **Archive Terminal** — a holographic interface inside the Capsuleer Combat Archive.

### Root: Historical Archives

```
╔═══════════════════════════════════════════════════╗
║   CONCORD CAPSULEER COMBAT ARCHIVE — TERMINAL     ║
╠═══════════════════════════════════════════════════╣
║                                                   ║
║   HISTORICAL ARCHIVES                             ║
║                                                   ║
║   ├─ Empire Wars                                  ║
║   │   ├─ ✓ Minmatar Rebellion (YC 110–112)      ║
║   │   │   └─ Elder Fleet Campaign [PLAY]          ║
║   │   ├─ ✓ Caldari–Gallente War (YC 110)        ║
║   │   │   └─ Orbital Skirmish [PLAY]              ║
║   │   └─ ○ Amarr Succession (YC 117)            ║
║   │       └─ [LOCKED — Purchase Archive Access]   ║
║   │                                               ║
║   ├─ Pirate Conflicts                             ║
║   │   ├─ ○ Sansha Incursions (YC 113–present)    ║
║   │   └─ ○ Guristas Campaigns (YC 108–115)      ║
║   │                                               ║
║   ├─ Major Invasions                              ║
║   │   ├─ ✓ Triglavian Invasion (YC 122)         ║
║   │   │   └─ EDENCOM Defense [PLAY]               ║
║   │   └─ ○ Drifter Emergence (YC 118)            ║
║   │                                               ║
║   └─ Abyssal Phenomena                            ║
║       ├─ ✓ Abyssal Depths Recon (YC 122)        ║
║       │   └─ Pocket Survival [PLAY]               ║
║       └─ ○ Kybernaut Raids (YC 125)             ║
║                                                   ║
║   [SIMULATION SETTINGS]  [LEADERBOARDS]         ║
║   [CAPSULEER PROFILE]    [LOG OUT]              ║
╚═══════════════════════════════════════════════════╝
```

**Visual:** Dark terminal aesthetic. Green and amber text. Occasional scan-line flicker. When a campaign is selected, a brief holographic wireframe of the relevant faction's logo rotates before transitioning to load screen.

---

## In-Game Voice Lines

### Mission Start

> **ARCHIVE:** "Loading historical engagement... Battle of Caldari Prime. Authenticity: 99.84%. Fleet records verified. Capsuleer interface online."

> **ARCHIVE:** "Mission parameters loaded. Objective: survive and neutralize hostile forces. Historical outcome is fixed. Your performance is not."

### Difficulty Selection

> **ARCHIVE (Newbro):** "Simulation fidelity reduced. Recommended for initial calibration."
> **ARCHIVE (Veteran):** "Standard historical parameters. Recommended for experienced capsuleers."
> **ARCHIVE (Ace):** "Maximum fidelity. Casualties expected. Historical accuracy prioritized over survivability."
> **ARCHIVE (Carebear / Easy):** "Training simulation. No permanent record will be kept."

### Mission Complete

> **ARCHIVE:** "Simulation complete. Historical record appended. Performance evaluation: [RATING]."

> **ARCHIVE:** "Capsuleer performance exceeds archived baseline. Recommend next archive: [SUGGESTED MISSION]."

### Death / Retry

> **ARCHIVE:** "Neural link severed. Clone activation in progress..."

> **ARCHIVE:** "Simulation reset. Remember: history cannot be changed. Only survived."

> **ARCHIVE:** "Analyzing failure telemetry. Adjusting parameters for next attempt."

### Boss Encounter Start

> **ARCHIVE:** "High-threat entity detected in historical record. Authenticity verification required."

> **ARCHIVE:** "Warning: classified engagement. This data was recovered from partial fleet logs. Expect incomplete telemetry."

### Boss Defeated

> **ARCHIVE:** "Historical outcome confirmed. Enemy command element neutralized. Fleet records updated."

### Unlock Notification

> **ARCHIVE:** "New archive decrypted. Access granted to: [CAMPAIGN NAME]. Historical context: [ONE-LINE SUMMARY]."

---

## Loading Screens

Each loading screen displays three rotating facts from EVE lore relevant to the upcoming mission:

**Example — Caldari Prime mission:**

```
> LOADING HISTORICAL RECORD...
>
> Did you know?
> Caldari Prime was originally a temperate world before
> orbital bombardment during the Gallente-Caldari War
> rendered much of it uninhabitable.
>
> Historical accuracy: 99.84%
> Fleet records: VERIFIED
> Capsuleer sync: ONLINE
```

---

## Post-Mission Debrief Screen

```
╔═══════════════════════════════════════════════════╗
║         ARCHIVE DEBRIEF — MISSION COMPLETE        ║
╠═══════════════════════════════════════════════════╣
║                                                   ║
║   Historical Engagement: Battle of Caldari Prime ║
║   Authenticity Rating: 99.91%                     ║
║                                                   ║
║   CAPSULEER PERFORMANCE                             ║
║   ├─ Enemies neutralized:     47                  ║
║   ├─ Accuracy:                78%                 ║
║   ├─ Chain multiplier:        12.4x               ║
║   ├─ Damage taken:            LOW                 ║
║   └─ Survival time:           4:32                ║
║                                                   ║
║   RANKING: A-GRADE                                ║
║   (Top 12% of archived simulations)               ║
║                                                   ║
║   [NEXT MISSION]  [REPLAY]  [ARCHIVE MENU]        ║
╚═══════════════════════════════════════════════════╝
```

---

## Faction-Specific Archive Intros

### Caldari–Gallente War

> **ARCHIVE:** "The Gallente-Caldari War is one of the longest-running conflicts in New Eden's recorded history. What began as a trade dispute escalated into total war. You are about to experience one of its defining moments. Choose your perspective."

### Minmatar Rebellion / Elder Fleet

> **ARCHIVE:** "YC 110. The Elder Fleet emerges from the Great Wildlands. For centuries, the Minmatar people suffered under Amarr enslavement. This reconstruction is based on recovered tribal fleet logs. Some data remains classified."

### Triglavian Invasion

> **ARCHIVE:** "YC 122. The Triglavian Collective breaches abyssal boundaries into New Eden proper. EDENCOM was formed in response. This reconstruction uses classified EDENCOM battle telemetry. Access level: restricted."

### Abyssal Depths

> **ARCHIVE:** "YC 122. Capsuleer reconnaissance into abyssal pocket space. Survival rates in original deployments were statistically negligible. This simulation will test whether modern capsuleer training has improved those odds."

---

## Endless Mode Framing

> **ARCHIVE:** "Stress-test mode initialized. Historical parameters extrapolated beyond recorded endpoints. This scenario did not occur. It is a simulation of what *could have* occurred had the conflict continued. Capsuleer endurance evaluation in progress."

---

## Integration Notes

### What Exists Now (v2.0)
- Vertical slice mode (`--vertical-slice` CLI flag)
- Main menu system (Bevy UI)
- Mission select screen
- Difficulty select (Newbro / Veteran / Ace / Carebear)
- Score persistence (`SaveData`)
- High score display

### What Needs to Change
1. **Main menu text** — Replace "PLAY VERTICAL SLICE" with "Access Archive: Caldari–Gallente War"
2. **Mission start** — Add Archive voice line (can be text-only initially)
3. **Loading screens** — Add lore fact rotation (requires new UI screen or overlay)
4. **Death screen** — Replace "GAME OVER" with "Neural link severed. Simulation reset."
5. **Victory screen** — Add "Historical accuracy" and "Archived simulations" language
6. **Unlock notifications** — Frame as "Archive decrypted"

### Priority
- **P0**: Menu text updates (pure copy changes, no new systems)
- **P1**: Death/victory screen copy updates
- **P2**: Voice line system (if audio budget allows)
- **P3**: Loading screen lore rotation

---

## Document History

- 2026-07-25 — Initial narrative framework distilled from user conversation

**This document is append-only.** Changes require a note in `CHANGELOG.md`.
