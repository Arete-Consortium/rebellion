# Rebellion

An EVE-inspired arcade space shooter built in Rust with Bevy 0.15. Fly recognizable faction ships, dodge enemy fire, fight phased bosses, and chase scores through close-range combat.

Rebellion is a **free portfolio and community project**. Playable releases
and future campaigns will remain free. Contributions, playtesting and
constructive feedback are welcome.

The current finishing target is the **Caldari–Gallente three-mission vertical slice**: a complete route from the menu through escalating encounters to a results screen. The codebase also contains Elder Fleet, Triglavian Invasion, Abyssal Depths, and alternate survival modes. Their presence is not a claim that every route is release-ready.

## Run on desktop

**Controller required.** The [controller-only checkpoint](docs/CONTROLLER_ONLY_PLAYTEST_2026-09-08.md) records the current Apple Silicon Mac and Intel/AMD Ubuntu candidates. For Linux runtime libraries, downloads and source packaging, see the [Linux playtest instructions](docs/LINUX_PLAYTEST.md).

Install a current stable [Rust toolchain](https://rust-lang.org/tools/install/) and your platform's compiler tools. The manifest declares Rust 1.85 or later. On macOS, install Xcode Command Line Tools; Linux builds also need ALSA, udev, XKB, and Wayland development libraries (see the CI workflow).

Run from this repository's root so the game can find `assets/`.

```sh
cargo run --locked --release
```

For a double-clickable macOS candidate package, use:

```sh
./scripts/package-macos-playtest.sh
open dist/Rebellion.app
```

The package places `assets/` in `Rebellion.app/Contents/Resources/assets` and configures
runtime launch to resolve assets from that folder automatically.

Choose **PLAY**, select the Minmatar–Amarr or Caldari–Gallente chapter, then pick your faction, difficulty and hull. A first build takes time because Bevy compiles the rendering and audio stack. The packager refuses to overwrite an existing app; supply a new output directory as its fourth argument for subsequent candidates.

Fixed controller layout (Xbox button names):

| Action | Input |
| --- | --- |
| Move / aim | Left stick / right stick |
| Fire / hull ability | RT / LT |
| Thrust / dodge | LB / RB |
| Previous / next ammunition | D-pad left / right |
| Interact / overload | A / Y |
| Pause | Menu |
| Menu navigation / confirm / back | Left stick or D-pad / A / B |

Connect a controller, release the sticks/buttons, then press and release A to continue. Losing the controller freezes play until you reconnect and acknowledge it. Keyboard, mouse and touch do not control player builds. The Controls screen shows the fixed layout. See the [controller playtest guide](docs/CONTROLLER_PLAYTEST.md) for physical-device checks. For a test run that keeps existing saves separate:

```sh
REBELLION_HOME=/tmp/rebellion-playtest cargo run --locked --release
```

## Verify changes

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets -- -D warnings
cargo bench --locked --bench game_systems
```

Headless tests cover simulation and selected gameplay systems. They do not establish rendered frame rate, controller feel, audio quality, or a complete human playthrough.

## Browser build

The project supports WebAssembly. Install the `wasm32-unknown-unknown` Rust target and a `wasm-bindgen-cli` version matching `wasm-bindgen` in `Cargo.lock`, then use `build-wasm.sh`. Serve `web/` over local HTTP to test it. Browser build tooling still needs a reproducibility pass; see the completion plan before distributing a build.

## Continue development

- [Production plan, milestones and prioritized backlog](docs/PRODUCTION_PLAN.md)
- [Current Mac and Linux candidates: controller-only play](docs/CONTROLLER_ONLY_PLAYTEST_2026-09-08.md)
- [Hull ability fixes and three-mission probes](docs/HULL_ABILITY_PLAYTEST_2026-09-08.md)
- [Previous booster combat checkpoint](docs/BOOSTER_COMBAT_PLAYTEST_2026-09-08.md)
- [Approved classic booster artwork](docs/CLASSIC_BOOSTER_PLAYTEST_2026-09-08.md)
- [Previous booster artwork and HUD checkpoint](docs/BOOSTER_PLAYTEST_2026-09-08.md)
- [Transport objectives and clearer shields](docs/TRANSPORT_PLAYTEST_2026-09-08.md)
- [Save recovery and audio checkpoint](docs/PLAYTEST_2026-09-08.md)
- [Chapter and sprite checkpoint](docs/MAC_PLAYTEST_REPORT.md)
- [Repair baseline and current verification](docs/COMPLETION_PLAN.md)
- [Code and dependency review](docs/CODE_REVIEW.md)
- [Gameplay and controls review](docs/GAMEPLAY_REVIEW.md)
- [Performance findings and measurement plan](docs/PERFORMANCE_AUDIT.md)
- [Existing vertical slice definition](docs/VERTICAL_SLICE.md)
- [Game positioning and faction identity](docs/PRODUCT_POSITIONING.md)
- [Ship registry tools](docs/SHIP_REGISTRY_PACKAGE.md) and [migration plan](docs/SHIP_REGISTRY_REFACTOR_PLAN.md)
- [Preserved ship-review versions and recovery tool](docs/ship-review/README.md)

Use [Arete-Consortium/rebellion](https://github.com/Arete-Consortium/rebellion) for current source. Its description still points to an older archived `arcade/eve-rebellion` snapshot; the standalone repository contains the later gameplay and test work. The previous root README described only the ship-registry package and has been preserved under `docs/`.

Code is distributed under the repository's [MIT license](LICENSE). Asset provenance is tracked separately in the ship-registry work.
