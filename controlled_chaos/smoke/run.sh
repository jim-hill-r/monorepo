#!/usr/bin/env bash
# Plays a script (default smoke/smoke.script) in the debug build under a
# virtual display, and records that display. Screenshots and smoke.mp4 go in
# target/smoke. Exits with the game's code: 0 passed, 1 a check failed,
# 101 the game panicked. Needs xvfb-run; ffmpeg is optional (no video without
# it). Run it with `mise run smoke` from controlled_chaos/.
set -euo pipefail

script="${1:-smoke/smoke.script}"
out="target/smoke"
size="1280x720" # Bevy's default window size

rm -rf "$out"
mkdir -p "$out"
cargo build

xvfb-run -a -s "-screen 0 ${size}x24" bash -c '
  script="$1" out="$2" size="$3"
  recorder=""
  if command -v ffmpeg >/dev/null; then
    ffmpeg -loglevel error -y -f x11grab -video_size "$size" -framerate 30 \
      -i "$DISPLAY" -pix_fmt yuv420p "$out/smoke.mp4" </dev/null &
    recorder=$!
  else
    echo "smoke: ffmpeg not found, so no video"
  fi
  status=0
  ./target/debug/controlled_chaos --script "$script" --screenshots "$out" || status=$?
  if [ -n "$recorder" ]; then
    kill -INT "$recorder" 2>/dev/null || true
    wait "$recorder" || true
  fi
  exit "$status"
' _ "$script" "$out" "$size"
