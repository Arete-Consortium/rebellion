#!/usr/bin/env python3
"""Verify shipped Mac, Linux, Windows and web archives without running them."""

import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import tarfile
import zipfile


spec = importlib.util.spec_from_file_location("manifest", Path(__file__).with_name("playtest-manifest.py"))
manifest = importlib.util.module_from_spec(spec)
spec.loader.exec_module(manifest)

TARGETS = {"aarch64-apple-darwin", "x86_64-unknown-linux-gnu",
           "x86_64-pc-windows-msvc", "wasm32-unknown-unknown"}


def check_archive(archive, evidence):
    if archive.name.endswith(".tar.gz"):
        container = tarfile.open(archive, "r:gz")
        names = [m.name for m in container.getmembers() if m.isfile()]
        read = lambda name: container.extractfile(name).read()
    else:
        container = zipfile.ZipFile(archive)
        names = [m.filename for m in container.infolist() if not m.is_dir()]
        read = container.read
    with container:
        manifests = [n for n in names if n.endswith("/PLAYTEST-MANIFEST.json")
                     or n == "PLAYTEST-MANIFEST.json"]
        if len(manifests) != 1 or len(names) != len(set(names)):
            raise ValueError(f"Expected one unambiguous package manifest in {archive}")
        manifest_name = manifests[0]
        data = read(manifest_name)
        metadata = json.loads(data)
        target = metadata["target"]
        if target not in TARGETS:
            raise ValueError(f"Unexpected package target: {target}")
        manifest_path = evidence / f"{target}.json"
        with manifest_path.open("xb") as output:
            output.write(data)
        metadata = manifest.read_manifest(manifest_path)
        prefix = manifest_name.removesuffix("PLAYTEST-MANIFEST.json") + "assets/"
        actual = {n.removeprefix(prefix): hashlib.sha256(read(n)).hexdigest()
                  for n in names if n.startswith(prefix)
                  and Path(n).name != ".DS_Store" and not Path(n).name.startswith("._")}
        if actual != metadata["asset_files"]:
            raise ValueError(f"Archive assets do not match its manifest: {archive}")
        return metadata, {"target": target, "archive": archive.name,
                          "archive_sha256": manifest.file_hash(archive),
                          "asset_files_verified": len(actual)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("artifacts", type=Path)
    parser.add_argument("evidence", type=Path)
    args = parser.parse_args()
    args.evidence.mkdir(parents=True, exist_ok=True)
    try:
        archives = sorted(args.artifacts.rglob("*.tar.gz")) + sorted(args.artifacts.rglob("*.zip"))
        manifests, packages = [], []
        for archive in archives:
            metadata, package = check_archive(archive, args.evidence)
            manifests.append(metadata)
            packages.append(package)
        if len(manifests) != len(TARGETS) or {m["target"] for m in manifests} != TARGETS:
            raise ValueError("Expected exactly one Mac, Linux, Windows and web package")
        manifest.compare_manifests(manifests, require_clean=True)
        result = {"status": "passed", "source_commit": manifests[0]["source_commit"],
                  "shared_source_sha256": manifests[0]["shared_source_sha256"],
                  "assets_sha256": manifests[0]["assets_sha256"], "packages": packages,
                  "scope": "Shared source and packaged assets; physical playtests remain separate"}
    except (OSError, ValueError, KeyError, tarfile.TarError, zipfile.BadZipFile) as error:
        (args.evidence / "result.json").write_text(json.dumps({"status": "failed", "error": str(error)}, indent=2) + "\n")
        parser.exit(1, f"{error}\n")
    (args.evidence / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
