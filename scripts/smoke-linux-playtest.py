#!/usr/bin/env python3
"""Launch a freshly extracted package under an existing X11/Wayland display.

Use xvfb-run for CI. This verifies native startup/assets with an isolated muted
save; it is not a GPU-performance, audio, controller or human gameplay test.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


def main():
    archive = Path(sys.argv[1]).resolve()
    evidence = Path(sys.argv[2]).resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="rebellion-linux-smoke-") as temp:
        workspace = Path(temp)
        subprocess.run(["tar", "-xzf", str(archive), "-C", temp], check=True)
        package, = workspace.glob("Rebellion-linux-*")
        subprocess.run(["sha256sum", "--check", "--quiet", "SHA256SUMS"], cwd=package, check=True)
        libraries = subprocess.run(["ldd", str(package / "rebellion")], text=True,
                                   capture_output=True, check=True).stdout
        (evidence / "libraries.txt").write_text(libraries)
        if "not found" in libraries:
            raise RuntimeError("Package has unresolved shared libraries")
        profile = workspace / "isolated-save"
        profile.mkdir()
        (profile / "save.json").write_text(json.dumps({
            "stage_progress": [], "unlocked_ships": [], "lifetime_credits": 0,
            "high_scores": [], "settings": {
                "master_volume": 0, "sfx_volume": 0, "music_volume": 0}}))
        environment = dict(os.environ, REBELLION_HOME=str(profile), RUST_LOG="info")
        # The launcher/executable must discover their assets, without Cargo or
        # a source-checkout override accidentally making this check pass.
        environment.pop("BEVY_ASSET_ROOT", None)
        environment.pop("CARGO_MANIFEST_DIR", None)
        log_path = evidence / "startup.log"
        with log_path.open("w") as log:
            process = subprocess.Popen([str(package / "play.sh")], cwd="/tmp",
                                       env=environment, stdout=log, stderr=subprocess.STDOUT)
            try:
                deadline = time.monotonic() + 45
                initialized_at = None
                while time.monotonic() < deadline and process.poll() is None:
                    output = log_path.read_text()
                    if "Rebellion initialized!" in output and initialized_at is None:
                        initialized_at = time.monotonic()
                    if initialized_at and time.monotonic() - initialized_at >= 10:
                        break
                    time.sleep(0.25)
                output = log_path.read_text()
                result = {
                    "initialized": initialized_at is not None,
                    "alive_after_startup": process.poll() is None and initialized_at is not None
                        and time.monotonic() - initialized_at >= 10,
                    "panic": "panicked" in output,
                    "missing_assets": "Path not found:" in output,
                    "checksums_verified": True,
                    "shared_libraries_resolved": True,
                    "isolated_save": True,
                }
                (evidence / "smoke-result.json").write_text(json.dumps(result, indent=2) + "\n")
                print(json.dumps(result))
                if not result["alive_after_startup"] or result["panic"] or result["missing_assets"]:
                    raise RuntimeError("Linux startup check failed; inspect startup.log")
            finally:
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()


if __name__ == "__main__":
    main()
