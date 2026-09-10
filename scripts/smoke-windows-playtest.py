#!/usr/bin/env python3
"""Verify a fresh Windows ZIP, then check native startup in a graphical session.

The muted synthetic save and startup observation do not qualify controller
hardware, sound, performance, or a complete human playthrough. --verify-only
checks package integrity on any host without claiming native startup.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import stat
import struct
import subprocess
import sys
import tempfile
import time
import zipfile


NAME = "Rebellion-windows-x86_64"
MANIFEST_TOOL = Path(__file__).resolve().with_name("playtest-manifest.py")


def sha(path):
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def relative_path(name):
    path = PurePosixPath(name)
    if not name or not path.parts or path.is_absolute() or ".." in path.parts or "\\" in name or ":" in name:
        raise ValueError(f"Invalid package path: {name}")
    return path


def verify_and_extract(archive, workspace):
    checksum = Path(str(archive) + ".sha256").read_text(encoding="utf-8-sig").strip()
    expected, name = checksum.split("  ", 1)
    if name != archive.name or not re.fullmatch(r"[a-fA-F0-9]{64}", expected):
        raise ValueError("Invalid archive checksum file")
    if sha(archive) != expected.lower():
        raise ValueError("Archive checksum mismatch")
    with zipfile.ZipFile(archive) as bundle:
        seen = set()
        for member in bundle.infolist():
            path = relative_path(member.filename)
            key = str(path).casefold()
            if path.parts[0] != NAME or key in seen:
                raise ValueError(f"Unexpected or duplicate ZIP member: {member.filename}")
            if stat.S_ISLNK(member.external_attr >> 16):
                raise ValueError(f"Package must not contain symlinks: {member.filename}")
            seen.add(key)
        bundle.extractall(workspace)
    package = workspace / NAME
    expected_files = {}
    for line in (package / "SHA256SUMS").read_text(encoding="utf-8-sig").splitlines():
        digest, name = line.split("  ", 1)
        path = relative_path(name)
        if name in expected_files or not re.fullmatch(r"[a-fA-F0-9]{64}", digest):
            raise ValueError(f"Invalid packaged checksum: {name}")
        expected_files[str(path)] = digest.lower()
    actual_files = {
        path.relative_to(package).as_posix(): sha(path)
        for path in package.rglob("*")
        if path.is_file() and path.relative_to(package).as_posix() != "SHA256SUMS"
    }
    if not expected_files or actual_files != expected_files:
        raise ValueError("Extracted file inventory/checksums do not match SHA256SUMS")
    binary = package / "rebellion.exe"
    data = binary.read_bytes()
    if len(data) < 64 or data[:2] != b"MZ":
        raise ValueError("Package does not contain a Windows executable")
    pe = struct.unpack_from("<I", data, 0x3C)[0]
    if pe < 64 or pe + 26 > len(data) or data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("Invalid Windows executable header")
    if struct.unpack_from("<H", data, pe + 4)[0] != 0x8664 or struct.unpack_from("<H", data, pe + 24)[0] != 0x20B:
        raise ValueError("Package executable is not Windows x86_64")
    manifest_path = package / "PLAYTEST-MANIFEST.json"
    subprocess.run([
        sys.executable, str(MANIFEST_TOOL), "verify", "--manifest", str(manifest_path),
        "--assets", str(package / "assets"),
    ], check=True)
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest["target"] != "x86_64-pc-windows-msvc":
        raise ValueError("Package content manifest has the wrong target")
    return package, manifest


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("--verify-only", action="store_true")
    args = parser.parse_args()
    if os.name != "nt" and not args.verify_only:
        parser.error("Native startup requires Windows; use --verify-only for integrity checks")
    archive = args.archive.resolve()
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    log_path = evidence / "startup.log"
    result_path = evidence / "smoke-result.json"
    for path in (log_path, result_path):
        if path.exists():
            parser.error(f"Use fresh evidence paths; existing file: {path}")
    result = {"status": "failed", "native_startup_checked": False}
    error = None
    try:
        with tempfile.TemporaryDirectory(prefix="rebellion-windows-smoke-") as temp:
            workspace = Path(temp)
            package, manifest = verify_and_extract(archive, workspace)
            result.update({
                "archive_sha256": sha(archive), "checksums_verified": True,
                "packaged_assets_verified": True, "target": manifest["target"],
                "shared_source_sha256": manifest["shared_source_sha256"],
                "assets_sha256": manifest["assets_sha256"],
            })
            if args.verify_only:
                result["status"] = "integrity_passed"
            else:
                profile = workspace / "isolated-save"
                profile.mkdir()
                (profile / "save.json").write_text(json.dumps({
                    "stage_progress": [], "unlocked_ships": [], "lifetime_credits": 0,
                    "high_scores": [], "settings": {
                        "master_volume": 0, "sfx_volume": 0, "music_volume": 0,
                    },
                }), encoding="utf-8")
                unrelated = workspace / "unrelated-working-directory"
                unrelated.mkdir()
                environment = dict(os.environ, REBELLION_HOME=str(profile), RUST_LOG="info")
                environment.pop("BEVY_ASSET_ROOT", None)
                environment.pop("CARGO_MANIFEST_DIR", None)
                with log_path.open("w", encoding="utf-8") as log:
                    process = subprocess.Popen(
                        [str(package / "rebellion.exe")], cwd=unrelated, env=environment,
                        stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT,
                    )
                    result.update({"native_startup_checked": True, "owned_pid": process.pid})
                    try:
                        deadline = time.monotonic() + 45
                        initialized_at = None
                        while time.monotonic() < deadline and process.poll() is None:
                            output = log_path.read_text(encoding="utf-8", errors="replace")
                            if "Rebellion initialized!" in output and initialized_at is None:
                                initialized_at = time.monotonic()
                            if initialized_at is not None and time.monotonic() - initialized_at >= 10:
                                break
                            time.sleep(0.25)
                        output = log_path.read_text(encoding="utf-8", errors="replace")
                        result.update({
                            "initialized": initialized_at is not None,
                            "alive_after_startup": process.poll() is None and initialized_at is not None
                            and time.monotonic() - initialized_at >= 10,
                            "panic": bool(re.search(r"panicked|fatal runtime error", output, re.I)),
                            "missing_assets": bool(re.search(r"Path not found:|Asset not found:|Failed to load asset", output, re.I)),
                            "isolated_save": True, "unrelated_working_directory": True,
                            "startup_exit_code": process.poll(),
                        })
                        if not result["alive_after_startup"] or result["panic"] or result["missing_assets"]:
                            raise RuntimeError("Windows startup check failed; inspect startup.log")
                    finally:
                        if process.poll() is None:
                            process.terminate()
                            try:
                                process.wait(timeout=5)
                            except subprocess.TimeoutExpired:
                                process.kill()
                                process.wait(timeout=5)
                        result["owned_process_closed"] = process.poll() is not None
                result["status"] = "passed"
    except (OSError, ValueError, RuntimeError, KeyError, struct.error, zipfile.BadZipFile, subprocess.SubprocessError) as failure:
        error = failure
        result["error"] = str(failure)
    result_path.write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2))
    if error is not None:
        raise SystemExit(1)


if __name__ == "__main__":
    main()
