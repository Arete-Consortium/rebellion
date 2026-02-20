# EVE Rebellion

[![CI](https://github.com/Arete-Consortium/eve_rebellion_rust/actions/workflows/ci.yml/badge.svg)](https://github.com/Arete-Consortium/eve_rebellion_rust/actions)
[![Release](https://img.shields.io/github/v/release/Arete-Consortium/eve_rebellion_rust)](https://github.com/Arete-Consortium/eve_rebellion_rust/releases)
[![Play Now](https://img.shields.io/badge/Play%20Now-GitHub%20Pages-blue?logo=github)](https://arete-consortium.github.io/eve_rebellion_rust)
[![Play on itch.io](https://img.shields.io/badge/Play-itch.io-FA5C5C?logo=itch.io)](https://aretedriver.itch.io/eve-rebellion)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows%20%7C%20Web-purple.svg)]()

**EVE Rebellion** is a collection of arcade shooters set in the EVE Online universe. Experience pivotal moments in New Eden's history through fast-paced vertical shooter gameplay.

## Campaigns

### Elder Fleet Invasion (YC110)
*Minmatar vs Amarr — 13 stages*

Rise from a rookie pilot in a rusty Rifter to an ace liberator in a Jaguar. Join the Elder Fleet as they emerge from decades of hiding to free the Minmatar people from centuries of Amarr slavery.

### Battle of Caldari Prime (YC115)
*Caldari vs Gallente — 5 missions + Endless Mode*

Choose your side in the battle that changed Caldari Prime forever. Fight as Caldari State defending Shiigeru, or Gallente Federation liberating the planet. Includes **Shiigeru Nightmare** — an endless survival mode aboard the dying Titan.

### Triglavian Invasion *(In Development)*
*EDENCOM vs Triglavian Collective — 9 missions*

Defend New Eden from the Triglavian invasion, or embrace the Flow and fight for Pochven. Choose EDENCOM to protect the empires, or join the Collective to claim systems for Pochven.

---

## Features

- **Vertical Shooter Action** — Fast-paced shmup gameplay with EVE Online ships and lore
- **Multiple Campaigns** — Story-driven missions across different eras of New Eden
- **Four Factions** — Minmatar, Amarr, Caldari, Gallente (Triglavian coming soon)
- **Heat & Combo System** — Push your weapons to the limit for score multipliers
- **Salt Miner Mode** — Fill your meter with proximity kills to unleash devastating power
- **Ship Abilities** — Faction-specific active abilities with cooldowns
- **Ship Progression** — Unlock faction ships as you advance
- **The Last Stand** — Defend Shiigeru as a fixed-platform titan battle
- **Endless Mode** — High-score survival with escalating difficulty
- **Authentic EVE Visuals** — Ships with faction color tints and parallax starfield
- **Procedural Audio** — Dynamic soundtrack and sound effects generated at runtime
- **Layer-Based Damage** — Shield ripples, armor sparks, hull fire with screen shake
- **Powerup Rarity System** — Common to Epic tiers with orbital particles and glow effects
- **Active Buff Visuals** — Shield bubbles, speed lines, damage auras while buffs active
- **Low Health Warning** — Pulsing red vignette when health is critical
- **Steam Deck Support** — Auto-detected profiles with tuned deadzones and back button mapping

## Controls

### Keyboard
- **WASD / Arrow Keys** - Move
- **Space** - Fire
- **B** - Activate Salt Miner (when meter full)
- **E** - Activate Ship Ability
- **Shift** - Barrel Roll (i-frames)

### Controller (Xbox/PlayStation/Steam Deck)
- **Left Stick** - Move
- **Right Stick** - Aim and Fire (twin-stick mode)
- **Right Trigger** - Activate Ship Ability
- **Y / Triangle** - Activate Salt Miner
- **Right Bumper** - Barrel Roll
- **A / X** - Context Action

#### Steam Deck Back Buttons
- **L4** - Previous Ammo Type
- **L5** - Speed Boost
- **R4** - Next Ammo Type
- **R5** - Quick Rocket

## Building

Requires Rust 1.75+ and Bevy 0.15.

```bash
cargo build --release
cargo run --release
```

## Project Structure

```
eve_rebellion_rust/
├── src/
│   ├── main.rs           # Entry point, plugin registration
│   ├── core/             # Game states, events, resources, factions
│   ├── entities/         # Player, enemies, bosses, projectiles, collectibles
│   ├── systems/          # Game logic (collision, scoring, spawning, effects)
│   ├── ui/               # HUD, menus, capacitor, backgrounds
│   ├── assets/           # Asset loading
│   └── games/            # Campaign modules (Bevy plugins)
│       ├── elder_fleet/          # Minmatar vs Amarr (13 stages)
│       ├── caldari_gallente/     # Caldari vs Gallente (5 missions + Nightmare)
│       ├── abyssal_depths/       # Abyssal Deadspace expansion
│       └── triglavian_invasion/  # Triglavian Invasion (in development)
├── assets/               # Sprites, icons (EVE Image Server cache)
├── config/               # JSON configuration
│   ├── enemies_amarr.json    # Amarr enemy definitions
│   ├── bosses_campaign.json  # 13-boss campaign structure
│   └── dialogue_elder.json   # Elder mentor dialogue
├── web/                  # WASM build files
└── docs/
    └── NARRATIVE_DESIGN.md   # Story bible and design notes
```

## Credits

- **EVE Online** is a trademark of CCP hf.
- Ship images provided via CCP's Image Server under their community use guidelines
- This is a fan project, not affiliated with or endorsed by CCP

## License

This project is licensed under the MIT License.

---

*"History is written by those who fly into the fire."*
