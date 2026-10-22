#!/bin/sh
set -e
# Paths
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${BV_BIN:-$ROOT/crates/bv-server/target/debug/blockverse}"
PLUGIN="${BV_PLUGIN:-$ROOT/target/debug/libbv_example_plugin.so}"
for f in "$BIN" "$PLUGIN"; do
  [ -f "$f" ] || { echo "Missing: $f"; exit 1; }
done
# Run
WORK="$(mktemp -d)"
mkdir -p "$WORK/plugins"
cp "$PLUGIN" "$WORK/plugins/"
cd "$WORK"
printf 'fakejoin Steve\nfakechat 1 hello everyone\nfakechat 1 this has a badword\nlist\nstatus\nstop\n' \
  | BV_NO_SETUP=1 BV_BIND=127.0.0.1:19199 "$BIN" > out.txt 2>&1
cat out.txt
# Checks
grep -q "Loaded plugin RustHello" out.txt
grep -q "Welcome to the server, Steve" out.txt
grep -q "cancelled by a plugin" out.txt
grep -q "Players online: 1/" out.txt
grep -q "Goodbye" out.txt
echo "SMOKE TEST PASSED"
