# CLAUDE.md — eve_rebellion_rust

## Project Overview

EVE Online arcade shooter suite - Rust/Bevy rewrite

## Current State

- **Version**: 1.9.0
- **Language**: Rust
- **Files**: 276 across 6 languages
- **Lines**: 53,098

## Architecture

```
eve_rebellion_rust/
├── .cargo/
├── .github/
│   └── workflows/
├── assets/
│   ├── audio/
│   ├── backgrounds/
│   ├── fonts/
│   ├── models/
│   ├── powerups/
│   ├── ships/
│   └── sprites/
├── benches/
├── config/
├── docs/
│   └── harvested_from_python/
├── games/
│   └── caldari_gallente/
├── platforms/
│   ├── linux/
│   ├── macos/
│   ├── web/
│   └── windows/
├── src/
│   ├── assets/
│   ├── audio/
│   ├── campaigns/
│   ├── core/
│   ├── entities/
│   ├── esi/
│   ├── games/
│   ├── systems/
│   └── ui/
├── web/
│   └── assets/
├── .gitignore
├── CHANGELOG.md
├── CLAUDE.md
├── CONTRIBUTING.md
├── Cargo.lock
├── Cargo.toml
├── LICENSE
├── README.md
├── build-wasm.sh
├── colorize_sprites.sh
```

## Tech Stack

- **Language**: Rust, Python, Shell, TypeScript, JavaScript, HTML
- **Framework**: bevy, rust
- **Package Manager**: cargo
- **Linters**: clippy
- **Test Frameworks**: cargo test
- **CI/CD**: GitHub Actions

## Coding Standards

- **Naming**: snake_case
- **Line Length (p95)**: 74 characters

## Anti-Patterns (Do NOT Do)

- Do NOT commit secrets, API keys, or credentials
- Do NOT skip writing tests for new code
- Do NOT use `any` type — define proper type interfaces
- Do NOT use `var` — use `const` or `let`
- Do NOT use `.unwrap()` in production code — use proper error handling
- Do NOT use `unsafe` without a safety comment
- Do NOT clone when a reference will do
- Do NOT use `os.path` — use `pathlib.Path` everywhere
- Do NOT use bare `except:` — catch specific exceptions
- Do NOT use mutable default arguments
- Do NOT use `print()` for logging — use the `logging` module

## Dependencies

### Core
- bevy_egui
- serde
- serde_json
- rand
- fastrand
- image

## Domain Context

### Key Models/Classes
- `Ability`
- `AbilityActivatedEvent`
- `AbilityAura`
- `AbilityEffectParticle`
- `AbilityEffectType`
- `AbilityEffects`
- `AbilityEndedEvent`
- `AbilityIndicatorContainer`
- `AbilityIndicatorFill`
- `AbilityIndicatorText`
- `AbilityKeyHint`
- `AbilityPlugin`
- `AbilityType`
- `AbyssalDepthsPlugin`
- `AbyssalEnemyText`

### Domain Terms
- Abyssal Deadspace
- Activate Salt Miner
- Activate Ship Ability
- Active Buff Visuals
- Arrow Keys
- Barrel Roll
- Based Damage
- Building Requires Rust
- CCP
- CI

### Enums/Constants
- `Ability`
- `AbilityEffectType`
- `AbilityType`
- `AbyssalRoom`
- `Achievement`
- `Act`
- `AmmoType`
- `Armor`
- `BackButtonAction`
- `BackgroundShipClass`

### Outstanding Items
- **TODO**: we could test for more things here, like `Set`s and `Map`s. (`web/eve_rebellion.js`)

## AI Skills

**Installed**: 122 skills in `~/.claude/skills/`
- `a11y`, `accessibility-checker`, `agent-teams-orchestrator`, `align-debug`, `api-client`, `api-docs`, `api-tester`, `apple-dev-best-practices`, `arch`, `backup`, `brand-voice-architect`, `build`, `changelog`, `ci`, `cicd-pipeline`
- ... and 107 more

**Recommended bundles**: `full-stack-dev`

**Recommended skills** (not yet installed):
- `full-stack-dev`

## Git Conventions

- Commit messages: Conventional commits (`feat:`, `fix:`, `docs:`, `test:`, `refactor:`)
- Branch naming: `feat/description`, `fix/description`
- Run tests before committing
