#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

TARGET="${1:-aarch64-apple-darwin}"
PROFILE="${2:-release}"
APP_NAME="${3:-Rebellion}"
OUTPUT_DIR="${4:-dist}"
BINARY="target/${TARGET}/${PROFILE}/rebellion"
VERSION="${VERSION:-$(rg -m 1 '^version\s*=' Cargo.toml | sed -E 's/.*"([^"]+)"/\1/')}"

if [[ "${PROFILE}" != "release" && "${PROFILE}" != "debug" ]]; then
  echo "[package] unsupported profile '${PROFILE}' (expected 'debug' or 'release')"
  exit 1
fi

if [[ "${PROFILE}" == "release" ]]; then
  PROFILE_FLAG="--release"
else
  PROFILE_FLAG=""
fi

# Bundle paths must remain within the selected output directory.
if [[ ! "$APP_NAME" =~ ^[A-Za-z0-9][A-Za-z0-9\ _-]*$ ]]; then
  echo "[package] APP_NAME must be a simple application name" >&2
  exit 1
fi

mkdir -p "$OUTPUT_DIR"
DESTINATION="$OUTPUT_DIR/${APP_NAME}.app"
if [[ -e "$DESTINATION" || -e "${DESTINATION}.zip" ]]; then
  echo "[package] destination already exists; choose a new name or output directory: $DESTINATION" >&2
  exit 1
fi

# Always ask Cargo to validate freshness; an existing binary may predate edits.
cargo build --offline --locked ${PROFILE_FLAG} --target "$TARGET"

# Sign outside Documents/Finder's metadata tracking, then publish the complete bundle.
STAGING_DIR="$(mktemp -d /private/tmp/rebellion-package.XXXXXX)"
trap 'rm -rf "$STAGING_DIR"' EXIT
APP_DIR="$STAGING_DIR/${APP_NAME}.app"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"

cp -X "$BINARY" "$APP_DIR/Contents/MacOS/$APP_NAME"
chmod +x "$APP_DIR/Contents/MacOS/$APP_NAME"
cp -RX assets "$APP_DIR/Contents/Resources/"
python3 scripts/playtest-manifest.py write --target "$TARGET" \
  --output "$APP_DIR/Contents/Resources/PLAYTEST-MANIFEST.json"
python3 scripts/playtest-manifest.py verify \
  --manifest "$APP_DIR/Contents/Resources/PLAYTEST-MANIFEST.json" \
  --assets "$APP_DIR/Contents/Resources/assets"

cat > "$APP_DIR/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleDisplayName</key>
  <string>$APP_NAME</string>
  <key>CFBundleExecutable</key>
  <string>$APP_NAME</string>
  <key>CFBundleIdentifier</key>
  <string>com.arete.rebellion</string>
  <key>CFBundleName</key>
  <string>$APP_NAME</string>
  <key>CFBundleVersion</key>
  <string>$VERSION</string>
  <key>CFBundleShortVersionString</key>
  <string>$VERSION</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>NSHighResolutionCapable</key>
  <true/>
  <key>NSPrincipalClass</key>
  <string>NSApplication</string>
</dict>
</plist>
PLIST

cat > "$APP_DIR/Contents/PkgInfo" <<'PKG'
APPLREBL
PKG

if command -v codesign >/dev/null; then
  # Finder metadata on source assets must not enter a signed bundle.
  xattr -cr "$APP_DIR"
  codesign --force --deep --sign - "$APP_DIR"
  codesign --verify --deep --strict "$APP_DIR"
fi
plutil -lint "$APP_DIR/Contents/Info.plist"
# Keep a sealed copy before a synced output directory can attach Finder metadata.
ditto -c -k --norsrc --noextattr --keepParent "$APP_DIR" "${DESTINATION}.zip"
if command -v codesign >/dev/null; then
  mkdir -p "$STAGING_DIR/verified"
  ditto -x -k "${DESTINATION}.zip" "$STAGING_DIR/verified"
  codesign --verify --deep --strict "$STAGING_DIR/verified/${APP_NAME}.app"
fi
mv "$APP_DIR" "$DESTINATION"
echo "Packaged app: $DESTINATION"
echo "Clean archive: ${DESTINATION}.zip"
echo "Run: open '$DESTINATION'"
