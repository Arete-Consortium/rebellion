# Rebellion on Ubuntu / Linux

The Intel/AMD package targets **64-bit Ubuntu 22.04 or newer**. It includes the game, all runtime artwork/audio, a launcher, source/compiler information and checksums. Rust is not needed to play the downloaded package. Other Linux distributions may work with equivalent libraries; they need their own playtest.

## Download and play

1. Open the repository's [Linux Playtest runs](https://github.com/Arete-Consortium/rebellion/actions/workflows/linux-playtest.yml), select the latest successful run for `codex/linux-playtest`, and download **Rebellion-linux-x86_64** under Artifacts. GitHub requires sign-in to download Actions artifacts; these downloads expire after 30 days.
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

Use **Play → chapter → faction → difficulty → hull**. WASD moves, Space fires, IJKL aims, and Escape pauses. Xbox-compatible controllers use the game's existing gamepad support; detection, reconnect, rumble and mapping still need checking on your machine.

## Build from the repository on Ubuntu

Check out `codex/linux-playtest` to get the current playtest work. Install a current stable Rust toolchain and these development libraries:

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

When reporting a problem, include `BUILD-INFO.txt`, your Ubuntu version, GPU, display session (X11 or Wayland), selected chapter/faction/hull, and what happened. Capture launch output with:

```sh
RUST_LOG=info ./Rebellion-linux-x86_64/play.sh > rebellion-linux.log 2>&1
```

Try the complete Caldari/Gallente three-mission route first. Check booster pickup/effect readability, boss health depletion, sound levels, pause/resume and controller behavior. See [the combat checkpoint](https://github.com/Arete-Consortium/rebellion/blob/codex/linux-playtest/docs/BOOSTER_COMBAT_PLAYTEST_2026-09-08.md) for the matching gameplay changes and remaining balance work.
