#!/bin/sh
# Paths
DIR="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$DIR/bvmc/report.txt"
mkdir -p "$DIR/bvmc"
# Report
{
  echo "== BlockVerse report =="
  date
  uname -a
  rustc --version 2>/dev/null
  echo
  echo "== settings =="
  cat "$DIR/bvmc/server.properties" "$DIR/bvmc/blockverse.properties" 2>/dev/null
  echo
  echo "== latest log (last 200 lines) =="
  tail -n 200 "$DIR/bvmc/logs/latest.log" 2>/dev/null
  echo
  echo "== previous log (last 100 lines) =="
  tail -n 100 "$DIR/bvmc/logs/previous.log" 2>/dev/null
} > "$OUT"
# Copy
if [ -d "$HOME/storage/downloads" ]; then
  cp "$OUT" "$HOME/storage/downloads/blockverse-report.txt"
  echo "Report copied to Downloads/blockverse-report.txt"
fi
echo "Report written to $OUT"
