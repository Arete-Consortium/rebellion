# Browser playtest

Rebellion requires a controller. This package contains the same game source and
canonical assets as the native playtest packages. The build manifest records the
source and asset identity; a successful build alone does not qualify a browser or
physical controller.

## Start locally

Extract the ZIP, open a terminal in `Rebellion-web`, and run:

```sh
bash play.sh
```

On macOS, `play.command` starts the same server. Python 3 must be installed.
Open `http://127.0.0.1:8080` in a browser with WebGL2 enabled. Keep the terminal
running; press Ctrl+C there to stop. If port 8080 is in use, run `bash play.sh 8081`
and use the printed address. Opening `index.html` as a file will not work.

On Windows, from the extracted directory run
`py -3 -m http.server 8080 --bind 127.0.0.1`, then open the same address.

Connect a controller and press a button so the browser can expose it to the game.
Release the sticks and buttons, then press and release A at the game's connection
prompt. There is no separate keyboard or mouse start screen. The browser may
require a click in the game to allow sound; that click does not control gameplay.
See `CONTROLLER_PLAYTEST.md` for the full layout and disconnect checks.

Browser saves belong to the browser profile and site address. They are separate
from native saves; use the same local address and port for repeat sessions.

## Acceptance checks

Record the browser/version, operating system, controller model, and wired or
wireless connection. Check controller acknowledgement, both sticks, triggers,
pause, disconnect/reconnect, sound, and a full mission/result/continue loop.
Test Caldari, Gallente, Minmatar, and Amarr. Confirm results retain the best chain,
combat time, and personal best comparison after a pause and mission transition.
Reload to verify the browser's score persistence. Compare the manifest source and
asset fingerprints with the native package before reporting parity.

## Rebuild from the repository

Install Rust's `wasm32-unknown-unknown` target and the `wasm-bindgen-cli` version
recorded for `wasm-bindgen` in `Cargo.lock`. The build script prints the exact
installation command if the CLI is missing or mismatched.

```sh
bash scripts/package-web-playtest.sh dist/web-playtest-NEW
```

The script uses the locked `wasm-release` profile, copies fresh canonical assets,
verifies `PLAYTEST-MANIFEST.json`, and writes `Rebellion-web.zip` with checksums.
Existing packages are never overwritten. Set `CARGO_NET_OFFLINE=true` to require
cached dependencies. Set `WASM_BINDGEN` to an explicit CLI path if needed.
For a browser folder without a ZIP, use `bash build-wasm.sh dist/web-build-NEW`.
