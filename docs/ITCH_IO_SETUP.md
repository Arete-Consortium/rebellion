# itch.io Project Setup — Rebellion

**Project URL:** `aretedriver.itch.io/rebellion`  
**Status:** Draft (ready to create manually on itch.io)  
**Last updated:** 2026-07-25

---

## 1. Basic Details

| Field | Value |
|-------|-------|
| **Title** | Rebellion |
| **Project URL** | `rebellion` → `aretedriver.itch.io/rebellion` |
| **Kind of project** | `HTML` (primary) + downloadable binaries |
| **Genre** | **Shooter** |
| **Made with** | Rust, Bevy |
| **Tags** | `shooter`, `arcade`, `action`, `sci-fi`, `space`, `controller`, `score-attack`, `boss-fight`, `open-source` |
| **Release status** | In development |
| **Pricing** | Free (donations optional) |

---

## 2. Cover Image

- **File:** `assets/icon.svg` or a 630×500 PNG render of the title screen
- **Alt text:** "Rebellion title screen — CONCORD Capsuleer Combat Archive"
- **Recommendation:** Dark background with green terminal text. The title "REBELLION" in large monospace font, with the subtitle "Historical Reconstruction Program" underneath.

---

## 3. Short Description (Displayed on itch.io Cards)

```
An arcade action game set in a sci-fi universe. Fly iconic ships through historical military simulations. 10-minute sessions. Controller-friendly.
```

**Character count:** 174 / 250 max ✅

---

## 4. Full Description (itch.io Markdown)

```markdown
# REBELLION

**The CONCORD Capsuleer Combat Archive reconstructs the defining conflicts of New Eden using authenticated battlefield telemetry, fleet records, and historical archives.**

You are a capsuleer accessing this archive.

Your objective is not to rewrite history.
Your objective is to survive it.

---

## What Is This?

Rebellion is an arcade action game asking: *What if the defining moments of New Eden were recreated as a polished, replayable, controller-friendly experience?*

Not an MMO. A complementary product for the gaps an MMO cannot fill:
- **10-minute play sessions**
- **Fast arcade combat**
- **Couch / controller-friendly gameplay**
- **Replayable action campaigns**
- **Pick-up-and-play experiences**

---

## Current Build: Vertical Slice

### Caldari–Gallente War (Missions 1–3)
- 3 waves of escalating intensity
- Boss encounters with phase transitions
- Score chasing and chain multipliers
- Ship selection and upgrade progression
- 20–30 minute self-contained experience

---

## Authenticity Principle

Every ship behaves like pilots expect:

| Faction | Combat Identity |
|---------|-----------------|
| **Caldari** | Missiles, shields, range control |
| **Gallente** | Drones, hybrid blasters, armor |
| **Minmatar** | Mobility, projectiles, improvisation |
| **Amarr** | Lasers, armor, sustained pressure |

---

## Controls

| Input | Action |
|-------|--------|
| Arrow Keys / Left Stick | Move |
| Z / A Button | Fire |
| X / B Button | Ability |
| Space / Start | Pause |
| ESC / Select | Menu |

Supports keyboard, Xbox, PlayStation, and Steam Deck controllers.

---

## Open Source

Rebellion is open source (MIT). Built with Rust and Bevy Engine.

[Source code on GitHub](https://github.com/Arete-Consortium/rebellion)

---

## Roadmap

- [x] Vertical slice (Missions 1–3)
- [ ] Missions 4–5 + T3 destroyers
- [ ] Elder Fleet campaign (9 missions)
- [ ] Triglavian Invasion campaign
- [ ] Abyssal Depths survival mode
- [ ] Full upgrade tree + achievements
- [ ] Steam Deck + controller polish

---

## System Requirements

| | Minimum |
|---|---|
| **OS** | Windows 10, macOS 12, Ubuntu 22.04, or a modern browser |
| **CPU** | Any x64 processor from the last 10 years |
| **RAM** | 4 GB |
| **GPU** | Anything supporting WebGL2 |
| **Storage** | ~60 MB |
| **Input** | Keyboard or gamepad |

The web build runs directly in your browser. No install required.

---

*Rebellion is an independent project. It is not affiliated with or endorsed by CCP Games.*
```

---

## 5. Upload Strategy

### Web Build (HTML)
- Upload via **butler** (not manual zip):
  ```bash
  butler push web/ aretedriver/rebellion:html5 --userversion v2.1.4
  ```
- This preserves file structure and enables delta updates
- Do NOT manually upload a zip for the HTML channel

### Native Builds
- **Linux:** `butler push linux-extracted/ aretedriver/rebellion:linux --userversion v2.1.4`
- **Windows:** `butler push windows-extracted/ aretedriver/rebellion:windows --userversion v2.1.4`
- **macOS:** `butler push macos-extracted/ aretedriver/rebellion:macos --userversion v2.1.4`

### File Layout for butler
```
web/
  ├── index.html
  ├── rebellion.js
  ├── rebellion_bg.wasm
  └── assets/
      ├── audio/
      ├── backgrounds/
      ├── factions/
      ├── fonts/
      ├── models/
      ├── powerups/
      ├── ships/
      └── sprites/
```

---

## 6. Screenshots (Recommended for itch.io Page)

Upload 3–5 screenshots in this order:

1. **Title screen** — Terminal aesthetic, "CONCORD Capsuleer Combat Archive"
2. **Gameplay 1** — Mission 1, Caldari frigate firing missiles, enemy explosion
3. **Gameplay 2** — Boss encounter, phase transition, HUD visible
4. **Gameplay 3** — Chain multiplier at 10×, screen full of projectiles
5. **Menu** — Archive terminal, faction ship selection

**Specs:** 1920×1080 or 1280×720 PNG, no watermarks

---

## 7. Visibility & Distribution

| Setting | Value | Rationale |
|---------|-------|-----------|
| **Visibility** | `Draft` initially | Verify build works before publicizing |
| **Release date** | Leave blank | In development, no hard date |
| **Pricing** | `$0.00` (Free) | Vertical slice is a demo / awareness tool |
| **Accept donations** | `Yes` | Tip jar only, not a revenue strategy |
| **Revenue share** | Default (itch.io 10%) | Standard |
| **Content warnings** | `Fantasy Violence`, `Simulated Gambling` (chain multiplier mechanics) | Optional, but honest |
| **Maturity** | `Everyone` or `Teen` | No blood, no language, explosions only |

### After Verification
Once the web build loads and plays correctly:
1. Switch visibility to `Restricted` → share link with testers
2. After 5–10 confirmed successful plays → switch to `Public`

---

## 8. Community Tab (Optional)

**Development log title:** "Archive Reconstruction Progress"

**First devlog:**
```
Vertical Slice Released

The Caldari–Gallente War reconstruction is now live.
3 missions. 2 boss encounters. 4 playable ships.
20–30 minutes of arcade action.

Feedback welcome. Ships behaving incorrectly? Let us know.
```

---

## 9. Analytics to Track

After publishing, monitor:
- Web vs. download ratio
- Average play duration (itch.io analytics)
- Drop-off point (Mission 1? Boss fight?)
- Controller vs. keyboard usage (in-game analytics if implemented)
- Most-played ship

---

## 10. Checklist Before Going Public

- [ ] itch.io project created at `aretedriver.itch.io/rebellion`
- [ ] Cover image uploaded
- [ ] Full description pasted
- [ ] Genre set to **Shooter**
- [ ] Tags added
- [ ] Web build pushed via butler
- [ ] Native builds pushed via butler (optional)
- [ ] Screenshots uploaded
- [ ] Visibility set to `Draft` for testing
- [ ] Played through Mission 1 in browser without errors
- [ ] Played through Mission 3 in browser without errors
- [ ] Visibility switched to `Public`
- [ ] Link shared in one EVE community (Reddit r/Eve, TweetFleet Slack, etc.)

---

## Document History

- 2026-07-25 — Initial draft based on PRODUCT_POSITIONING.md and vertical slice scope

**This document is append-only.** Changes require a note in `CHANGELOG.md`.
