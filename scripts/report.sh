#!/bin/sh
# Paths
DIR="$(cd "$(dirname "$0")/.." && pwd)"
DATA="$DIR/bvmc"
if [ -f "$DIR/server.properties" ]; then DATA="$DIR"; fi
OUT="$DATA/report.txt"
mkdir -p "$DATA"
# Report
{
  echo "== BlockVerse report =="
  date
  uname -a
  rustc --version 2>/dev/null
  echo
  echo "== settings =="
  cat "$DATA/server.properties" "$DATA/blockverse.properties" 2>/dev/null
  echo
  echo "== latest log (last 200 lines) =="
  tail -n 200 "$DATA/logs/latest.log" 2>/dev/null
  echo
  echo "== previous log (last 100 lines) =="
  tail -n 100 "$DATA/logs/previous.log" 2>/dev/null
} > "$OUT"
# Copy
if [ -d "$HOME/storage/downloads" ]; then
  cp "$OUT" "$HOME/storage/downloads/blockverse-report.txt"
  echo "Report copied to Downloads/blockverse-report.txt"
fi
echo "Report written to $OUT"
