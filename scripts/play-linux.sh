#!/usr/bin/env bash
# Works from another directory and when the extracted path contains spaces.
set -euo pipefail
APP_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$APP_DIR"
if [[ ! -x "$APP_DIR/rebellion" || ! -d "$APP_DIR/assets" ]]; then
  echo "Extract the complete Rebellion archive before running play.sh." >&2
  exit 1
fi
exec "$APP_DIR/rebellion" "$@"
