# Rebellion on Ubuntu / Linux

The Intel/AMD package targets **64-bit Ubuntu 22.04 or newer**. It includes the game, all runtime artwork/audio, a launcher, source/compiler information and checksums. Rust is not needed to play the downloaded package. Other Linux distributions may work with equivalent libraries; they need their own playtest.

**Controller required.** Matching Mac, Linux, Windows and browser candidates share their game source and runtime content. `PLAYTEST-MANIFEST.json` identifies those inputs, including the current run-results and stored-booster mechanics. Older archives may contain different mechanics or outdated instructions; choose all platform packages from the same successful Platform Playtest run.

## Download and play

1. Open the repository's [Platform Playtest runs](https://github.com/Arete-Consortium/rebellion/actions/workflows/linux-playtest.yml), select the successful run matching the checkpoint being tested, and download **Rebellion-linux-x86_64** under Artifacts. Its **Verify package parity** job must pass before treating the four packages as a matched set. GitHub requires sign-in; Actions downloads expire after 30 days.
2. Extract the downloaded artifact ZIP. It contains `Rebellion-linux-x86_64.tar.gz` and its checksum file.
3. In a terminal in that folder:

```sh
sha256sum --check Rebellion-linux-x86_64.tar.gz.sha256
tar -xzf Rebellion-linux-x86_64.tar.gz
./Rebellion-linux-x86_64/play.sh
```

Keep `rebellion`, `play.sh` and `assets/` together. The launcher works from any working directory, including paths containing spaces. No system-wide install or administrator access is needed to run the game. Use your GPU vendor's working Vulkan driver; Mesa Vulkan drivers support many Intel/AMD GPUs.

If Ubuntu reports missing audio/input/window libraries, install the runtime packages below. Ubuntu 24.04 and later may resolve `libasound2` as `libasound2t64`; use that package name if apt requests it.

```sh
sudo apt-get update
sudo apt-get install libasound2 libudev1 libxkbcommon0 libxkbcommon-x11-0 libwayland-client0 libwayland-cursor0 libwayland-egl1 libx11-6 libxrandr2 libxi6 libxcursor1 libxinerama1 libvulkan1 mesa-vulkan-drivers
```

Saves normally live in `~/.local/share/rebellion/` (or your configured XDG data directory). For an isolated playtest:

```sh
REBELLION_HOME="$HOME/rebellion-test-save" ./Rebellion-linux-x86_64/play.sh
```

Connect a controller, release the sticks/buttons, then press and release **A** to continue. Use **Play → chapter → faction → difficulty → hull**. The fixed layout uses Xbox-style button names:

| Action | Input |
| --- | --- |
| Move | Left stick |
| Aim and fire | Push right stick; center it to stop firing |
| Thrust / hull ability | RT / LT |
| Dodge / charged overload | RB + left-stick direction / LB |
| Previous / next ammunition | X / B or D-pad left / right; autocannon hulls |
| Select / use timed booster | D-pad up/down / Y |
| Interact / confirm | A |
| Pause / resume | Menu |
| Menu navigation / back | Left stick or D-pad / B |

Timed pickups store up to three doses each of Overclocker, Pyrolancea and X-Instinct. Select an occupied slot with D-pad up/down and press Y to use one dose. An empty or already-active selection spends nothing; holding Y cannot use another dose when the effect ends. Unused doses carry between missions, but death, a new run/hull selection or an explicit restart clears them. Repairs, capacitor and cooling pickups remain immediate.

Keyboard, mouse and touch do not control player builds. Disconnect freezes play: reconnect, release all controls, then press and release A again. An existing pause menu remains paused. Options → Controls shows the layout. Follow the [controller playtest guide](https://github.com/Arete-Consortium/rebellion/blob/codex/linux-playtest/docs/CONTROLLER_PLAYTEST.md) to check physical detection, reconnect, stick feel, sound and rumble on your machine.

## Build from the repository on Ubuntu

Use a separate clean checkout of `codex/linux-playtest` for source builds. To reproduce a candidate, select the source commit in its `PLAYTEST-MANIFEST.json` and the compiler in `BUILD-INFO.txt`. Install the recorded Rust toolchain and these development libraries:

The window, audio and input dependencies follow [Bevy 0.15's Linux setup](https://github.com/bevyengine/bevy/blob/v0.15.3/docs/linux_dependencies.md).

```sh
sudo apt-get install build-essential pkg-config libasound2-dev libudev-dev libxkbcommon-dev libwayland-dev libx11-dev libxrandr-dev libxi-dev libxcursor-dev libxinerama-dev
cargo fetch --locked --target x86_64-unknown-linux-gnu
bash scripts/package-linux-playtest.sh
./dist/linux-playtest/Rebellion-linux-x86_64/play.sh
```

For another candidate, pass a fresh output directory, such as `bash scripts/package-linux-playtest.sh dist/linux-test-2`. The packager refuses to overwrite an existing game/archive. It builds using the lockfile and cached dependencies, then bundles the runtime assets. Generated binaries stay in `dist/` and GitHub Actions artifacts rather than Git history.

## Verification and feedback

The workflow runs the gameplay regression suite, packages the release executable, extracts it in a fresh temporary folder, verifies checksums/shared libraries, and starts the real game with Xvfb/software Vulkan and a muted isolated save. The package is uploaded only after that check succeeds. The software-rendering check does not certify hardware performance, sound, controller support or human playability.

When reporting a problem, include `BUILD-INFO.txt` and `PLAYTEST-MANIFEST.json`, your Ubuntu version, GPU, display session (X11 or Wayland), selected chapter/faction/hull, and what happened. Capture launch output with:

```sh
RUST_LOG=info ./Rebellion-linux-x86_64/play.sh > rebellion-linux.log 2>&1
```

Try the complete Caldari/Gallente three-mission route first. Check stored-dose counts, D-pad selection and Y activation while dodging, effect/countdown readability, boss health depletion, sound levels, pause/resume and controller disconnect/reconnect. Confirm unused doses reach the next mission and an explicit restart clears them. Confirm results retain the best chain after the live combo expires, combat time pauses in menus, and terminal scores compare against the previous personal best. Complete human playthroughs and physical-device qualification remain open.
