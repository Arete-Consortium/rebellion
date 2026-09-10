#!/usr/bin/env python3
"""Identify shared gameplay inputs and verify the assets shipped on each platform.

Equal fingerprints establish source/content parity, not hardware acceptance.
Only renderer, storage, windowing and controller backend integration may differ.
"""

import argparse
import hashlib
import json
from pathlib import Path
import re
import subprocess


ROOT = Path(__file__).resolve().parent.parent
SCHEMA = 1


def file_hash(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_hash(files):
    # Names are POSIX-relative and bytes are canonical LF in Git on every host.
    return hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":"))
                          .encode("utf-8")).hexdigest()


def inventory(directory):
    if not directory.is_dir():
        raise ValueError(f"Missing input directory: {directory}")
    return {path.relative_to(directory).as_posix(): file_hash(path)
            for path in sorted(directory.rglob("*"))
            if path.is_file() and path.name != ".DS_Store" and not path.name.startswith("._")}


def shared_inputs(root):
    files = {}
    for name in ("src", "config"):
        files.update({f"{name}/{path}": digest for path, digest in inventory(root / name).items()})
    for path in sorted((root / "games").glob("*/config/*.json")):
        files[path.relative_to(root).as_posix()] = file_hash(path)
    for name in ("Cargo.toml", "Cargo.lock", ".cargo/config.toml"):
        files[name] = file_hash(root / name)
    return dict(sorted(files.items()))


def make_manifest(root, target):
    source = shared_inputs(root)
    assets = inventory(root / "assets")
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    status = subprocess.check_output(
        ["git", "status", "--porcelain", "--untracked-files=normal"], cwd=root, text=True)
    return {"schema_version": SCHEMA, "target": target, "source_commit": revision,
            "working_tree": "modified" if status else "clean",
            "shared_source_sha256": tree_hash(source), "assets_sha256": tree_hash(assets),
            "shared_source_files": source, "asset_files": assets}


def read_manifest(path):
    manifest = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(manifest, dict):
        raise ValueError(f"Invalid manifest object: {path}")
    if type(manifest.get("schema_version")) is not int or manifest["schema_version"] != SCHEMA:
        raise ValueError(f"Unsupported manifest schema: {path}")
    target = manifest.get("target")
    if not isinstance(target, str) or not target.strip():
        raise ValueError(f"Invalid target metadata: {path}")
    commit = manifest.get("source_commit")
    if not isinstance(commit, str) or re.fullmatch(r"[0-9a-fA-F]{40}", commit) is None:
        raise ValueError(f"Invalid source_commit metadata: {path}")
    if manifest.get("working_tree") not in ("clean", "modified"):
        raise ValueError(f"Invalid working_tree metadata: {path}")
    for field, digest in (("shared_source_files", "shared_source_sha256"),
                          ("asset_files", "assets_sha256")):
        files = manifest.get(field)
        if not isinstance(files, dict) or not files:
            raise ValueError(f"Invalid {field} fingerprint: {path}")
        for name, file_digest in files.items():
            if (not isinstance(name, str) or "\\" in name
                    or any(part in ("", ".", "..") for part in name.split("/"))
                    or (len(name) > 1 and name[1] == ":")):
                raise ValueError(f"Invalid {field} relative POSIX path: {path}")
            if (not isinstance(file_digest, str)
                    or re.fullmatch(r"[0-9a-f]{64}", file_digest) is None):
                raise ValueError(f"Invalid {field} file digest: {path}")
        if tree_hash(files) != manifest.get(digest):
            raise ValueError(f"Invalid {field} fingerprint: {path}")
    return manifest


def verify_assets(manifest, assets):
    actual = inventory(assets)
    expected = manifest["asset_files"]
    if actual != expected:
        missing = sorted(expected.keys() - actual.keys())
        extra = sorted(actual.keys() - expected.keys())
        changed = sorted(k for k in expected.keys() & actual.keys() if expected[k] != actual[k])
        raise ValueError(f"Packaged assets differ: missing={missing}, extra={extra}, changed={changed}")


def compare_manifests(manifests, require_clean=False):
    first = manifests[0]
    for manifest in manifests:
        for field in ("shared_source_sha256", "assets_sha256"):
            if manifest[field] != first[field]:
                raise ValueError(f"Platform parity failed for {manifest['target']}: {field}")
        if require_clean and (manifest["working_tree"] != "clean"
                              or manifest["source_commit"] != first["source_commit"]):
            raise ValueError("CI packages must share one clean source commit")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    write = commands.add_parser("write")
    write.add_argument("--target", required=True)
    write.add_argument("--output", type=Path, required=True)
    verify = commands.add_parser("verify")
    verify.add_argument("--manifest", type=Path, required=True)
    verify.add_argument("--assets", type=Path, required=True)
    compare = commands.add_parser("compare")
    compare.add_argument("--require-clean", action="store_true")
    compare.add_argument("manifests", nargs="+", type=Path)
    args = parser.parse_args()
    try:
        if args.command == "write":
            manifest = make_manifest(ROOT, args.target)
            # A package must be fresh; never silently replace its provenance.
            with args.output.open("x", encoding="utf-8", newline="\n") as output:
                json.dump(manifest, output, indent=2, sort_keys=True)
                output.write("\n")
            print(f"Wrote {args.output}: {manifest['shared_source_sha256']}")
        elif args.command == "verify":
            manifest = read_manifest(args.manifest)
            verify_assets(manifest, args.assets)
            print(f"Verified {len(manifest['asset_files'])} assets for {manifest['target']}")
        else:
            if len(args.manifests) < 2:
                raise ValueError("Compare at least two platform manifests")
            manifests = [read_manifest(path) for path in args.manifests]
            compare_manifests(manifests, args.require_clean)
            print(json.dumps({"status": "passed", "targets": [m["target"] for m in manifests],
                              "shared_source_sha256": manifests[0]["shared_source_sha256"],
                              "assets_sha256": manifests[0]["assets_sha256"]}, indent=2))
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        parser.exit(1, f"{error}\n")


if __name__ == "__main__":
    main()
