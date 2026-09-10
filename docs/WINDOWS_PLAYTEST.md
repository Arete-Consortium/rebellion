# Rebellion on Windows

The Windows playtest package targets **64-bit Intel/AMD Windows**. It contains `rebellion.exe`, runtime assets, this guide, the license, build information and checksums. Rust and Python are only needed to build or verify development packages. The packager uses the static MSVC runtime; normal Windows system libraries and a working graphics driver remain required.

This is the same controller-only game as the matching Mac/Linux candidate: both conflict chapters, the Caldari–Gallente three-mission slice, stored timed boosters and retained run statistics. Compare `PLAYTEST-MANIFEST.json` between candidates to confirm content parity. A shared source tree does not establish Windows startup, hardware performance or physical-controller support; record those checks on Windows before calling a package qualified.

## Extract and play

Use a Windows ZIP whose source and content manifest match the candidate being tested. In PowerShell, in the folder containing `Rebellion-windows-x86_64.zip` and its `.sha256` file:

```powershell
$Expected = ((Get-Content .\Rebellion-windows-x86_64.zip.sha256 -Raw).Trim() -split '\s+')[0]
$Actual = (Get-FileHash .\Rebellion-windows-x86_64.zip -Algorithm SHA256).Hash
if ($Actual -ne $Expected) { throw 'Archive checksum mismatch' }
Expand-Archive .\Rebellion-windows-x86_64.zip -DestinationPath .\rebellion-playtest
& .\rebellion-playtest\Rebellion-windows-x86_64\rebellion.exe
```

Keep `rebellion.exe` and `assets` together. You can also double-click the extracted executable; no system-wide installation is needed. The executable finds the bundled assets when launched from another working directory. Use a fresh destination when extracting a different candidate.

**Controller required.** Release the sticks/buttons, then press and release A to acknowledge connection. Select **Play → chapter → faction → difficulty → hull**. Controls use Xbox-style button names:

| Action | Input |
| --- | --- |
| Move / aim and fire | Left stick / push right stick |
| Stop primary fire | Center the right stick |
| Thrust / hull ability | RT / LT |
| Dodge / charged overload | RB + left-stick direction / LB |
| Previous / next ammunition | X / B or D-pad left / right; autocannon hulls |
| Select / use timed booster | D-pad up/down / Y |
| Interact / confirm | A |
| Pause / resume | Menu |
| Menu navigation / back | Left stick or D-pad / B |

Carry up to three doses each of Overclocker, Pyrolancea and X-Instinct. Y spends one selected dose per fresh press; empty or already-active selections spend nothing. Doses carry between missions, but death, a new run/hull selection or explicit restart clears them. Repairs, capacitor and cooling pickups remain immediate. Results retain the best chain and combat time after the live combo expires and compare terminal scores with the previous personal best.

Keyboard, mouse and touch do not control player builds. Disconnect freezes play; reconnect, release all controls, then press and release A again. An existing pause menu remains paused. The Controls screen shows the layout.

Use a separate save folder for testing:

```powershell
$env:REBELLION_HOME = Join-Path $env:TEMP ('rebellion-playtest-' + [Guid]::NewGuid().ToString('N'))
& .\rebellion-playtest\Rebellion-windows-x86_64\rebellion.exe
```

## Build and verify on Windows

Use the same source revision/content as the other platform candidates. Install Rust with the `x86_64-pc-windows-msvc` toolchain, Visual Studio Build Tools with C++ desktop build support and a Windows SDK, Python 3, and Git. In the repository root:

```powershell
rustup target add x86_64-pc-windows-msvc
cargo fetch --locked --target x86_64-pc-windows-msvc
powershell -NoProfile -File .\scripts\package-windows-playtest.ps1
python .\scripts\smoke-windows-playtest.py .\dist\windows-playtest\Rebellion-windows-x86_64.zip .\build\windows-smoke
```

If dependencies are already cached, the packager builds entirely offline with the lockfile. Pass `-OutputDir dist/windows-test-2` for a later package or `-Python C:\path\to\python.exe` to select Python. Existing output is never overwritten. Generated ZIPs and executables belong in ignored `dist` output or build artifacts.

The startup helper extracts into a fresh temporary directory, verifies every packaged file and the content manifest, launches the actual executable with a muted isolated save from an unrelated directory, and checks that initialization completes and the process remains alive. It closes only its owned process and writes startup evidence. A graphical Windows session is required. This is a startup check; physical controller behavior, sound and complete human runs remain separate.

Complete the Caldari/Kestrel and Gallente/Tristan three-mission routes on the Windows PC. Check pause/retry/reconnect, booster selection and activation while dodging, mission continuation, peak-chain retention and final personal-best comparison. Report Windows version, GPU/driver, controller/connection, chapter/faction/hull, `BUILD-INFO.txt`, `PLAYTEST-MANIFEST.json` and reproduction steps for failures. Preserve `startup.log` and `smoke-result.json` when the startup helper fails.
