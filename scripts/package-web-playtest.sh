#!/usr/bin/env bash
# Produce a local browser playtest folder and ZIP. Does not deploy or publish.
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"
if [[ $# -gt 1 || "${1:-}" == --help || "${1:-}" == -h ]]; then
  echo "Usage: $0 [OUTPUT_DIR]  (default: dist/web-playtest)"
  if [[ $# -gt 1 ]]; then exit 2; fi
  exit 0
fi
OUTPUT_DIR="${1:-dist/web-playtest}"
OUTPUT_DIR="$(python3 -c 'import os, sys; print(os.path.abspath(sys.argv[1]))' "$OUTPUT_DIR")"
mkdir -p "$OUTPUT_DIR"
OUTPUT_DIR="$(cd "$OUTPUT_DIR" && pwd)"
NAME="Rebellion-web"
DESTINATION="$OUTPUT_DIR/$NAME"
ARCHIVE="$DESTINATION.zip"
for path in "$DESTINATION" "$ARCHIVE" "$ARCHIVE.sha256"; do
  if [[ -e "$path" || -L "$path" ]]; then
    echo "Destination exists; choose a new output directory: $path" >&2
    exit 1
  fi
done
STAGING_DIR="$(mktemp -d "$OUTPUT_DIR/.rebellion-web-package.XXXXXX")"
trap 'rm -rf "$STAGING_DIR"' EXIT
PACKAGE="$STAGING_DIR/$NAME"
bash build-wasm.sh "$PACKAGE"
cp LICENSE "$PACKAGE/LICENSE"
cp docs/WEB_PLAYTEST.md "$PACKAGE/README.md"
cp docs/CONTROLLER_PLAYTEST.md "$PACKAGE/CONTROLLER_PLAYTEST.md"
cat > "$PACKAGE/play.sh" <<'SH'
#!/usr/bin/env bash
set -euo pipefail
PACKAGE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PORT="${1:-8080}"
if [[ ! "$PORT" =~ ^[1-9][0-9]{3,4}$ ]] || (( PORT < 1024 || PORT > 65535 )); then
  echo "Use a port between 1024 and 65535 (default: 8080)." >&2
  exit 1
fi
if ! command -v python3 >/dev/null 2>&1; then
  echo "Python 3 is required to serve this local browser build." >&2
  exit 1
fi
echo "Open http://127.0.0.1:$PORT in your browser. Press Ctrl+C here to stop."
exec python3 -m http.server "$PORT" --bind 127.0.0.1 --directory "$PACKAGE_DIR"
SH
chmod 755 "$PACKAGE/play.sh"
cp "$PACKAGE/play.sh" "$PACKAGE/play.command"
python3 scripts/playtest-manifest.py verify \
  --manifest "$PACKAGE/PLAYTEST-MANIFEST.json" --assets "$PACKAGE/assets"

# Python's ZIP writer is portable and retains the launcher executable bits.
python3 - "$PACKAGE" "$STAGING_DIR/$NAME.zip" <<'PY'
import hashlib
import pathlib
import sys
import zipfile

package = pathlib.Path(sys.argv[1])
archive = pathlib.Path(sys.argv[2])
files = sorted(path for path in package.rglob('*') if path.is_file())
with (package / 'SHA256SUMS').open('w') as sums:
    for path in files:
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        sums.write(f'{digest}  {path.relative_to(package).as_posix()}\n')
with zipfile.ZipFile(archive, 'w', compression=zipfile.ZIP_DEFLATED, compresslevel=9) as output:
    for path in sorted(path for path in package.rglob('*') if path.is_file()):
        output.write(path, (pathlib.Path(package.name) / path.relative_to(package)).as_posix())
with zipfile.ZipFile(archive) as check:
    invalid = check.testzip()
    if invalid:
        raise SystemExit(f'ZIP integrity check failed: {invalid}')
archive.with_suffix('.zip.sha256').write_text(
    f'{hashlib.sha256(archive.read_bytes()).hexdigest()}  {archive.name}\n'
)
PY
mv "$STAGING_DIR/$NAME.zip" "$ARCHIVE"
mv "$STAGING_DIR/$NAME.zip.sha256" "$ARCHIVE.sha256"
mv "$PACKAGE" "$DESTINATION"
echo "Browser playtest: $DESTINATION"
echo "Transfer archive: $ARCHIVE"
echo "Run: bash '$DESTINATION/play.sh'"
