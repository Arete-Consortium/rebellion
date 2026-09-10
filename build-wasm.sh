#!/usr/bin/env bash
# Build a fresh browser distribution without modifying the web/ source shell.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT_DIR"
if [[ $# -gt 1 || "${1:-}" == --help || "${1:-}" == -h ]]; then
  echo "Usage: $0 [OUTPUT_DIR]  (default: dist/web-build; must not exist)"
  echo "Requires Rust's wasm32-unknown-unknown target and the locked wasm-bindgen CLI."
  if [[ $# -gt 1 ]]; then exit 2; fi
  exit 0
fi
OUTPUT_DIR="${1:-dist/web-build}"
WASM_BINDGEN="${WASM_BINDGEN:-wasm-bindgen}"
for tool in cargo rustc python3; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    echo "Required build tool is missing: $tool" >&2
    exit 1
  fi
done

# Read the dependency lock, rather than installing a moving CLI version.
BINDGEN_VERSION="$(python3 - <<'PY'
import pathlib
import re

lock = pathlib.Path('Cargo.lock').read_text()
versions = []
for package in re.split(r'^\[\[package\]\]\s*$', lock, flags=re.MULTILINE):
    if re.search(r'^name = "wasm-bindgen"$', package, re.MULTILINE):
        match = re.search(r'^version = "([0-9]+\.[0-9]+\.[0-9]+)"$', package, re.MULTILINE)
        if match:
            versions.append(match.group(1))
if len(versions) != 1:
    raise SystemExit('Cargo.lock must contain exactly one wasm-bindgen version')
print(versions[0])
PY
)"
if ! command -v "$WASM_BINDGEN" >/dev/null 2>&1 \
    || [[ "$("$WASM_BINDGEN" --version)" != "wasm-bindgen $BINDGEN_VERSION" ]]; then
  echo "The browser bindings require wasm-bindgen $BINDGEN_VERSION from Cargo.lock." >&2
  echo "Install it with: cargo install --locked wasm-bindgen-cli --version $BINDGEN_VERSION" >&2
  echo "Or set WASM_BINDGEN to that version's executable." >&2
  exit 1
fi

OUTPUT_DIR="$(python3 -c 'import os, sys; print(os.path.abspath(sys.argv[1]))' "$OUTPUT_DIR")"
if [[ -e "$OUTPUT_DIR" || -L "$OUTPUT_DIR" ]]; then
  echo "Destination exists; choose a new output directory: $OUTPUT_DIR" >&2
  exit 1
fi
mkdir -p "$(dirname "$OUTPUT_DIR")"
STAGING_DIR="$(mktemp -d "$(dirname "$OUTPUT_DIR")/.rebellion-web-build.XXXXXX")"
trap 'rm -rf "$STAGING_DIR"' EXIT

# Respect CARGO_TARGET_DIR and Cargo configuration; never reuse a guessed path.
TARGET_DIR="$(cargo metadata --no-deps --format-version 1 --locked \
  | python3 -c 'import json, sys; print(json.load(sys.stdin)["target_directory"])')"
cargo build --locked --profile wasm-release --bin rebellion --target wasm32-unknown-unknown
"$WASM_BINDGEN" \
  --out-dir "$STAGING_DIR" \
  --out-name rebellion \
  --target web \
  "$TARGET_DIR/wasm32-unknown-unknown/wasm-release/rebellion.wasm"

cp web/index.html "$STAGING_DIR/index.html"
# A new directory prevents removed/renamed assets surviving an earlier build.
COPYFILE_DISABLE=1 cp -R assets "$STAGING_DIR/assets"
python3 scripts/playtest-manifest.py write \
  --target wasm32-unknown-unknown --output "$STAGING_DIR/PLAYTEST-MANIFEST.json"
python3 scripts/playtest-manifest.py verify \
  --manifest "$STAGING_DIR/PLAYTEST-MANIFEST.json" --assets "$STAGING_DIR/assets"
for required in index.html rebellion.js rebellion_bg.wasm PLAYTEST-MANIFEST.json; do
  if [[ ! -s "$STAGING_DIR/$required" ]]; then
    echo "Build output is missing or empty: $required" >&2
    exit 1
  fi
done
{
  echo "Rebellion browser playtest"
  echo "Target: wasm32-unknown-unknown"
  echo "Profile: wasm-release"
  echo "Built: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "Source: $(git rev-parse HEAD)"
  rustc --version
  "$WASM_BINDGEN" --version
} > "$STAGING_DIR/BUILD-INFO.txt"

mv "$STAGING_DIR" "$OUTPUT_DIR"
echo "Browser build: $OUTPUT_DIR"
echo "Serve locally: python3 -m http.server 8080 --bind 127.0.0.1 --directory '$OUTPUT_DIR'"
echo "Then open http://127.0.0.1:8080"
