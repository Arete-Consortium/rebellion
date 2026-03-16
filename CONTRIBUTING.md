# Contributing to Rebellion

Thank you for your interest in contributing! This document provides guidelines for contributing to the project.

## How to Contribute

### Reporting Issues

If you find a bug or have a suggestion:

1. Check if the issue already exists in [GitHub Issues](https://github.com/Arete-Consortium/eve_rebellion_rust/issues)
2. If not, create a new issue with:
   - Clear description of the problem or suggestion
   - Steps to reproduce (for bugs)
   - Your OS and Rust version
   - Any relevant error messages or screenshots

### Submitting Changes

1. **Fork and clone**
   ```bash
   git clone https://github.com/Arete-Consortium/eve_rebellion_rust.git
   cd eve_rebellion_rust
   ```

2. **Install system dependencies**

   Linux (Ubuntu/Debian):
   ```bash
   sudo apt-get install -y libasound2-dev libudev-dev libxkbcommon-dev libwayland-dev
   ```

3. **Build and run**
   ```bash
   cargo build
   cargo run
   ```

4. **Create a branch**
   ```bash
   git checkout -b feature/your-feature-name
   ```

5. **Make your changes**
   - Follow the existing code style
   - Add tests for new functionality
   - Update documentation as needed

6. **Run tests and linting**
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```

7. **Commit and push**
   ```bash
   git add .
   git commit -m "Description of changes"
   git push origin feature/your-feature-name
   ```

8. **Create a Pull Request**

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` and fix warnings
- Use meaningful variable and function names
- Add documentation comments for public items
- Avoid `unwrap()` in production code — use proper error handling or `.unwrap_or_default()`

## Testing

- Write tests for new functionality
- Ensure all existing tests pass
- Run tests with: `cargo test`

## Project Structure

```
eve_rebellion_rust/
├── src/
│   ├── main.rs              # Entry point, plugin registration
│   ├── core/                # Game states, resources, events, constants
│   │   ├── game_state.rs    # State machine (MainMenu → Playing → Victory/Death)
│   │   ├── resources.rs     # Shared game resources
│   │   ├── factions.rs      # Faction definitions (Minmatar, Amarr, Caldari, Gallente)
│   │   ├── campaign.rs      # Campaign progression tracking
│   │   ├── save.rs          # Save/load system
│   │   └── achievements.rs  # Achievement tracking
│   ├── entities/            # ECS components
│   │   ├── player.rs        # Player ship components and spawning
│   │   ├── enemy.rs         # Enemy types, behavior, AI
│   │   ├── boss.rs          # Boss entities with phase-based combat
│   │   ├── projectile.rs    # Weapons and projectile physics
│   │   ├── collectible.rs   # Powerups and pickups
│   │   ├── wingman.rs       # Allied escort ships
│   │   └── drone.rs         # Drone entities
│   ├── systems/             # Game logic (ECS systems)
│   │   ├── collision.rs     # Hit detection
│   │   ├── spawning.rs      # Wave and enemy spawning
│   │   ├── scoring_v2.rs    # Combo chains, style grades, salt miner
│   │   ├── effects.rs       # Visual effects (explosions, trails, particles)
│   │   ├── audio.rs         # Procedural sound generation
│   │   ├── music.rs         # Dynamic music system
│   │   ├── boss.rs          # Boss behavior logic
│   │   ├── joystick.rs      # Controller input (Xbox, PlayStation, Steam Deck)
│   │   ├── dialogue.rs      # Campaign dialogue system
│   │   ├── ability.rs       # Ship abilities with cooldowns
│   │   └── campaign.rs      # Campaign state transitions
│   ├── ui/                  # HUD and menus
│   │   ├── hud.rs           # In-game HUD overlay
│   │   ├── menu.rs          # Main menu, faction select
│   │   ├── capacitor.rs     # Capacitor wheel
│   │   ├── backgrounds.rs   # Parallax starfield rendering
│   │   └── transitions.rs   # Screen transitions
│   └── games/               # Campaign modules (Bevy plugins)
│       ├── elder_fleet/     # Elder Fleet Invasion (Minmatar vs Amarr, 13 stages)
│       ├── caldari_gallente/ # Battle of Caldari Prime (5 missions + Nightmare)
│       ├── abyssal_depths/  # Abyssal Deadspace (expansion)
│       └── triglavian_invasion/ # Triglavian Invasion (in development)
├── assets/                  # Sprites, icons
├── config/                  # JSON definitions (enemies, bosses, dialogue, stages)
├── web/                     # WASM build files and HTML shell
├── docs/                    # Design documents and narrative bible
└── Cargo.toml               # Dependencies (Bevy 0.15, bevy_egui)
```

## Building for Web (WASM)

```bash
./build-wasm.sh
cd web && python3 -m http.server 8080
```

## Areas for Contribution

### High Priority
- Triglavian Invasion campaign content (missions, dialogue, bosses)
- Unit tests for core systems (scoring, collision, spawning)
- Performance profiling and optimization
- Accessibility improvements

### Medium Priority
- Additional ship abilities per faction
- New enemy patterns and formations
- Tutorial system improvements
- Localization support

### Nice to Have
- Multiplayer support
- Level editor
- Replay system
- Additional faction campaigns (Pirate factions, CONCORD)

## Development Tips

### Adding a new campaign
1. Create a new module under `src/games/`
2. Implement the campaign as a Bevy `Plugin`
3. Register it in `src/games/mod.rs`
4. Add JSON config in `config/` for enemies, stages, dialogue
5. Add ship definitions in `ships.rs`

### Adding a new enemy type
1. Add the enemy definition to the appropriate JSON in `config/`
2. Implement spawn behavior in `src/systems/spawning.rs`
3. Add any special AI in `src/entities/enemy.rs`
4. Add visual effects in `src/systems/effects.rs`

### Adding a powerup
1. Define in `src/entities/collectible.rs`
2. Add collection logic in `src/systems/collision.rs`
3. Add visual effects in `src/systems/effects.rs`
4. Add sprite in `assets/powerups/`

## Questions?

- Open an issue for questions
- Check existing issues and discussions

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow

Thank you for contributing!
