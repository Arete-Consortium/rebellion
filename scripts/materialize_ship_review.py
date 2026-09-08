#!/usr/bin/env python3
"""Verify or reconstruct every original ship-review file from the archive index."""

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import shutil
import sys


PROJECT_ROOT = Path(__file__).resolve().parent.parent
DEFAULT_MANIFEST = PROJECT_ROOT / "docs" / "ship-review" / "manifest.json"


def relative_path(value):
    path = PurePosixPath(value)
    if path.is_absolute() or ".." in path.parts or not path.parts:
        raise ValueError(f"Expected a relative archive path: {value!r}")
    return path


def verify_archive(manifest_path):
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    if manifest.get("schema_version") != 1:
        raise ValueError("Unsupported ship-review manifest version")
    archive_root = manifest_path.parent.resolve()
    checked = {}
    files = []
    for original, entry in manifest["files"].items():
        original_path = relative_path(original)
        stored_path = relative_path(entry["stored_path"])
        source = (archive_root / stored_path).resolve()
        if not source.is_relative_to(archive_root):
            raise ValueError(f"Stored file escapes archive: {stored_path}")
        if source not in checked:
            contents = source.read_bytes()
            checked[source] = (len(contents), hashlib.sha256(contents).hexdigest())
        if checked[source] != (entry["bytes"], entry["sha256"]):
            raise ValueError(f"Integrity mismatch for {original}")
        files.append((original_path, source, entry))
    if len(files) != manifest["summary"]["original_file_count"]:
        raise ValueError("Original file count does not match manifest summary")
    if len(checked) != manifest["summary"]["stored_file_count"]:
        raise ValueError("Stored file count does not match manifest summary")
    return files, checked


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, default=DEFAULT_MANIFEST)
    parser.add_argument("--verify-only", action="store_true", help="Check every indexed hash without writing files")
    parser.add_argument("--output", type=Path, default=PROJECT_ROOT / "build" / "ship-review" / "restored")
    args = parser.parse_args()
    files, checked = verify_archive(args.manifest.resolve())
    print(f"Verified {len(files)} original files backed by {len(checked)} stored files.")
    if args.verify_only:
        return

    output = args.output.resolve()
    destinations = []
    # Validate the complete destination first; never overwrite different content.
    for original, source, entry in files:
        destination = output / original
        if not destination.resolve().is_relative_to(output):
            raise ValueError(f"Destination escapes output directory: {original}")
        if destination.exists():
            if not destination.is_file():
                raise ValueError(f"Destination is not a file: {destination}")
            digest = hashlib.sha256(destination.read_bytes()).hexdigest()
            if digest != entry["sha256"]:
                raise ValueError(f"Refusing to overwrite different content: {destination}")
        destinations.append((source, destination))
    for source, destination in destinations:
        destination.parent.mkdir(parents=True, exist_ok=True)
        if not destination.exists():
            shutil.copyfile(source, destination)
    print(f"Restored {len(files)} files under {output}")


if __name__ == "__main__":
    try:
        main()
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"Ship-review archive error: {error}", file=sys.stderr)
        sys.exit(1)
