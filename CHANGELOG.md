# Changelog

All notable changes to Rebellion will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.0] - 2026-04-04

### Added
- WASM audio: Full procedural SFX and music now works in browser (was completely silent)
- Universal WAV encoder replacing hound crate (works on both native and WASM)
- Gameplay analytics system tracking deaths by stage, ship popularity, difficulty distribution
- Data-driven enemy spawn factory (EnemyVariant enum + spawn_variant function)
- 106 new tests across wav_encoder, game_state, enemy types, spawn factory, collectibles, bosses, analytics (163 → 269 total)

### Changed
- Major codebase modularization (7 files split into 56 focused submodules):
  - menu.rs (5,167 lines) → 17 submodules
  - effects.rs (2,904 lines) → 12 submodules
  - caldari_gallente/mod.rs (3,159 lines) → 6 submodules
  - hud.rs (1,783 lines) → 9 submodules
  - enemy.rs (1,501 lines) → 5 submodules
  - audio.rs (1,355 lines) → 3 submodules
  - factions.rs (1,180 lines) → 4 submodules
- All spawn callsites migrated to spawn_variant() — 10 legacy wrappers removed
- Menu text contrast improved to WCAG AA compliance (4.5:1+ ratio)
- Minimum font size raised to 12px across all menus
- Bench profile panic setting fixed (was producing Cargo warning)

### Removed
- hound crate dependency (replaced by built-in WAV encoder)
- 10 redundant spawn_* wrapper functions (replaced by spawn_variant)

## [1.9.0] - 2025-01-26

### Added
- Visual effects: Powerup rarity system with orbital particles and glow scaling (Common/Uncommon/Rare/Epic)
- Visual effects: Layer-specific damage feedback (shield ripples, armor sparks, hull fire)
- Visual effects: Multi-phase pickup effects (flash, shockwave, particle burst)
- Visual effects: Buff expiration warnings (HUD pulsing, countdown text, screen edge glow)
- Visual effects: Active buff visuals (invuln shield bubble, overdrive speed lines, damage aura)
- Visual effects: Low health vignette warning (pulsing red border below 30% HP)
- Visual effects: Background ship traffic (multi-sprite silhouettes with engine glow)
- Controller: Steam Deck auto-detection with tuned deadzones (0.12/0.15)
- Controller: Back button mapping (L4/L5/R4/R5) for ammo cycling, boost, rockets
- Controller: Per-controller profiles (Steam Deck, Xbox, PlayStation)

### Changed
- Performance: Added particle caps for all effect systems to prevent lag
- Background ships now use detailed multi-sprite silhouettes instead of rectangles

## [1.8.0] - 2025-01-25

### Added
- Visual polish: enhanced explosions with shockwave rings, center flash, ember particles
- Visual polish: engine trails with dual-layer system (hot white core + faction color glow)
- Visual polish: parallax starfield background during gameplay
- Audio polish: dynamic pitch/volume variation for weapons, explosions, damage sounds
- Gameplay polish: extended chain timeout (2.0s → 2.5s)
- Gameplay polish: faster shield recharge (5 → 8/sec)
- Gameplay polish: salt miner meter decay grace period (1.5s)
- The Last Stand mode (fixed-platform titan defense)
- Ship ability system with cooldowns
- Drone entities and heat display
- Faction-aware dialogue system

### Changed
- Capacitor wheel moved to bottom-right corner, capacitor-style yellow dashes
- Binary size reduced 29% (75MB → 53MB) via profile optimizations

### Fixed
- Ship sprites display correctly with faction color tints
- Kestrel sprite rotation in Gallente chapter
- Remove dead code and compiler warnings
- Cargo fmt and clippy dead_code warning
- Prevent regular player/HUD spawn during Last Stand mode

## [1.5.1] - 2025-01-05

### Added
- CLAUDE.md project instructions
- itch.io badge to README
- Platform support documentation

### Fixed
- Clippy lint fixes (is_multiple_of, Range::contains)
- Cargo fmt formatting
- itch.io deploy continue-on-error

## [1.5.0] - 2025-01-04

### Added
- Automated itch.io deployment on release
- WASM build job in release workflow
- Linux, Windows, and Web builds pushed to itch.io

### Fixed
- CG boss projectiles not moving (use ProjectilePhysics)
- CG boss sprites (use faction-appropriate ship type IDs)
- Sprite rotation (bosses face player)
- Available sprites for CG bosses (no missing textures)

## [1.4.2] - 2025-01-03

### Added
- Combo timer bar below combo counter
- Settings persistence to disk (audio, screen shake, rumble)
- Rumble intensity slider in pause menu

## [1.4.1] - 2025-01-03

### Added
- Controller rumble/haptic feedback
- Pause menu volume sliders (Master, Music, SFX)
- Screen shake intensity slider
- Elder Fleet endless mode wave/boss announcements
- Nightmare mode announcements

### Fixed
- CG mission advancement timing

## [1.4.0] - 2025-01-02

### Added
- Critical Hit System with visual feedback
- Battle of Caldari Prime Campaign (5 missions)
- Shiigeru Nightmare Mode (endless survival)
- Mode select screen for Caldari
- Nightmare HUD with wave/time/kills/hull

## [1.3.0] - 2025-01-01

### Added
- Endless Mode survival gameplay
- Visual powerup status bar with timer bars
- Redesigned ship selection UI with stat bars
- More sound effects (missiles, waves, powerups)
- Salt Miner system redesign with meter

## [1.2.1] - 2024-12-31

### Added
- Web/WASM build support
- GitHub Pages deployment workflow

## [1.2.0] - 2024-12-30

### Added
- Initial release with Elder Fleet campaign
- 4 factions: Minmatar, Amarr, Caldari, Gallente
- 13-stage campaign with boss battles
- Procedural audio system
- Ship unlocks and progression

[Unreleased]: https://github.com/Arete-Consortium/rebellion/compare/v1.9.0...HEAD
[1.9.0]: https://github.com/Arete-Consortium/rebellion/compare/v1.8.0...v1.9.0
[1.8.0]: https://github.com/Arete-Consortium/rebellion/compare/v1.5.1...v1.8.0
[1.5.1]: https://github.com/Arete-Consortium/rebellion/compare/v1.5.0...v1.5.1
[1.5.0]: https://github.com/Arete-Consortium/rebellion/compare/v1.4.2...v1.5.0
[1.4.2]: https://github.com/Arete-Consortium/rebellion/compare/v1.4.1...v1.4.2
[1.4.1]: https://github.com/Arete-Consortium/rebellion/compare/v1.4.0...v1.4.1
[1.4.0]: https://github.com/Arete-Consortium/rebellion/compare/v1.3.0...v1.4.0
[1.3.0]: https://github.com/Arete-Consortium/rebellion/compare/v1.2.1...v1.3.0
[1.2.1]: https://github.com/Arete-Consortium/rebellion/compare/v1.2.0...v1.2.1
[1.2.0]: https://github.com/Arete-Consortium/rebellion/releases/tag/v1.2.0
