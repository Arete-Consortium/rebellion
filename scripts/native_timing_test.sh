#!/bin/bash
set -euo pipefail

REBELLION_DIR="/home/arete/projects/rebellion"
BINARY="$REBELLION_DIR/target/release/rebellion"
LOG_FILE="$REBELLION_DIR/native_timing_test.log"
HARD_TIMEOUT=1200  # 20 minutes max

# Clean up previous log
> "$LOG_FILE"

# Start Xvfb
Xvfb :99 -screen 0 1280x720x24 &
XVFB_PID=$!
sleep 2

echo "[native-timing] Starting game..."
DISPLAY=:99 "$BINARY" >> "$LOG_FILE" 2>&1 &
GAME_PID=$!

# Wait for window to appear
WINDOW_ID=""
for i in $(seq 1 30); do
    WINDOW_ID=$(xdotool search --pid "$GAME_PID" 2>/dev/null | head -1 || true)
    if [ -n "$WINDOW_ID" ]; then
        echo "[native-timing] Window found: $WINDOW_ID"
        break
    fi
    sleep 1
done

if [ -z "$WINDOW_ID" ]; then
    echo "[native-timing] FAIL: Window did not appear"
    kill "$GAME_PID" 2>/dev/null || true
    kill "$XVFB_PID" 2>/dev/null || true
    exit 1
fi

# Activate window and start itch mode flow
xdotool windowactivate "$WINDOW_ID"
sleep 0.5

# Press Space to select PLAY (itch mode auto-starts CG)
echo "[native-timing] Sending Space to start itch mode..."
xdotool key --window "$WINDOW_ID" space
sleep 2

# Press Space again to start from mission briefing
echo "[native-timing] Sending Space to start from briefing..."
xdotool key --window "$WINDOW_ID" space
sleep 1

# Bot loop: send random WASD + Space every ~600ms
echo "[native-timing] Bot started — random WASD + Space"
START_TIME=$(date +%s)
KEYS=(w a s d space)
CYCLE=0

while true; do
    ELAPSED=$(($(date +%s) - START_TIME))
    if [ "$ELAPSED" -ge "$HARD_TIMEOUT" ]; then
        echo "[native-timing] Hard timeout reached ($HARD_TIMEOUT s)"
        break
    fi

    # Send one random key
    KEY=${KEYS[$((CYCLE % 5))]}
    xdotool key --window "$WINDOW_ID" "$KEY"
    CYCLE=$((CYCLE + 1))

    # Log progress every 30s
    if [ "$((ELAPSED % 30))" -eq 0 ] && [ "$CYCLE" -gt 1 ]; then
        echo "[native-timing] ${ELAPSED}s elapsed, keys sent: $CYCLE"
    fi

    # Check for timer log
    if grep -q "SESSION TIMER" "$LOG_FILE" 2>/dev/null; then
        echo "[native-timing] Session timer log detected!"
        break
    fi

    sleep 0.5
done

# Capture final results
echo ""
echo "=== Native Timing Test Results ==="
echo "[native-timing] Total elapsed: $(($(date +%s) - START_TIME))s"
if grep -q "SESSION TIMER" "$LOG_FILE" 2>/dev/null; then
    grep "SESSION TIMER" "$LOG_FILE"
else
    echo "[native-timing] No SESSION TIMER log captured"
fi

# Show last 20 lines of log for context
echo "[native-timing] Last 20 log lines:"
tail -20 "$LOG_FILE"

# Cleanup
kill "$GAME_PID" 2>/dev/null || true
kill "$XVFB_PID" 2>/dev/null || true

echo "[native-timing] Done"
