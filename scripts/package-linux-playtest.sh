#!/usr/bin/env bash
# Build a portable Linux folder and archive. Run on the target Linux architecture.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"
if [[ "$(uname -s)" != Linux ]]; then
  echo "Build this package on Linux, or download the Linux Playtest workflow artifact." >&2
  exit 1
fi
case "$(uname -m)" in
  x86_64) TARGET=x86_64-unknown-linux-gnu; ARCH=x86_64 ;;
  aarch64) TARGET=aarch64-unknown-linux-gnu; ARCH=aarch64 ;;
  *) echo "Unsupported Linux architecture: $(uname -m)" >&2; exit 1 ;;
esac

OUTPUT_DIR="${1:-dist/linux-playtest}"
mkdir -p "$OUTPUT_DIR"
OUTPUT_DIR="$(cd "$OUTPUT_DIR" && pwd)"
NAME="Rebellion-linux-$ARCH"
DESTINATION="$OUTPUT_DIR/$NAME"
ARCHIVE="$DESTINATION.tar.gz"
for path in "$DESTINATION" "$ARCHIVE" "$ARCHIVE.sha256"; do
  if [[ -e "$path" ]]; then
    echo "Destination exists; choose a new output directory: $path" >&2
    exit 1
  fi
done

# Fetch first when online: cargo fetch --locked --target <target>.
# Packaging itself uses only the locked, locally cached dependencies.
cargo build --offline --locked --release --bin rebellion --target "$TARGET"
BINARY="target/$TARGET/release/rebellion"
if ! file "$BINARY" | grep -q 'ELF 64-bit'; then
  echo "Expected a 64-bit Linux executable: $BINARY" >&2
  exit 1
fi

STAGING_DIR="$(mktemp -d "$OUTPUT_DIR/.rebellion-package.XXXXXX")"
trap 'rm -rf "$STAGING_DIR"' EXIT
PACKAGE="$STAGING_DIR/$NAME"
mkdir -p "$PACKAGE"
install -m 755 "$BINARY" "$PACKAGE/rebellion"
cp -R assets "$PACKAGE/assets"
cp LICENSE "$PACKAGE/LICENSE"
cp docs/LINUX_PLAYTEST.md "$PACKAGE/README.md"
install -m 755 scripts/play-linux.sh "$PACKAGE/play.sh"
python3 scripts/playtest-manifest.py write --target "$TARGET" \
  --output "$PACKAGE/PLAYTEST-MANIFEST.json"
python3 scripts/playtest-manifest.py verify \
  --manifest "$PACKAGE/PLAYTEST-MANIFEST.json" --assets "$PACKAGE/assets"

# Record the source revision plus local changes and the exact compiler used.
{
  echo "Rebellion Linux playtest"
  echo "Target: $TARGET"
  echo "Built: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
  echo "Source: $(git rev-parse HEAD)"
  if [[ -n "$(git status --porcelain --untracked-files=normal)" ]]; then
    echo "Working tree: modified (this archive includes local changes)"
  else
    echo "Working tree: clean"
  fi
  rustc --version
  ldd --version | sed -n '1p'
} > "$PACKAGE/BUILD-INFO.txt"
(
  cd "$PACKAGE"
  find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS
  sha256sum --check --quiet SHA256SUMS
)
tar -C "$STAGING_DIR" -czf "$ARCHIVE" "$NAME"
(
  cd "$OUTPUT_DIR"
  sha256sum "$NAME.tar.gz" > "$NAME.tar.gz.sha256"
)
mv "$PACKAGE" "$DESTINATION"
echo "Packaged game: $DESTINATION"
echo "Transfer archive: $ARCHIVE"
echo "Run: $DESTINATION/play.sh"
