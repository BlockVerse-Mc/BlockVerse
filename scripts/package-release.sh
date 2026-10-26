#!/bin/sh
# Packages a built server into dist/ for GitHub Releases
set -eu

# Inputs
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BIN="${1:-}"
TARGET="${2:-}"
LABEL="${3:-}"
if [ -z "$BIN" ]; then
  for candidate in "$ROOT/crates/bv-server/target/release/blockverse" "$ROOT/crates/bv-server/target/debug/blockverse"; do
    if [ -x "$candidate" ]; then BIN="$candidate"; break; fi
  done
fi
[ -n "$BIN" ] && [ -x "$BIN" ] || { echo "No built server found. Build it first or run: sh scripts/package-release.sh <binary> <target>"; exit 1; }
if [ -z "$TARGET" ]; then
  case "$(uname -m)" in
    aarch64|arm64) ARCH=aarch64 ;;
    x86_64|amd64) ARCH=x86_64 ;;
    *) echo "Unknown CPU, pass the target name as the second argument"; exit 1 ;;
  esac
  case "${PREFIX:-}" in
    *com.termux*) TARGET="android-$ARCH" ;;
    *) TARGET="linux-$ARCH" ;;
  esac
fi

# Names
case "$TARGET" in
  windows-*) EXE="blockverse.exe" ;;
  *) EXE="blockverse" ;;
esac

# Stage
VERSION="$(grep -m1 '^version' "$ROOT/crates/bv-server/Cargo.toml" | cut -d'"' -f2)"
NAME="blockverse-${LABEL:-$VERSION}-$TARGET"
OUT="$ROOT/dist"
STAGE="$OUT/$NAME"
rm -rf "$STAGE"
mkdir -p "$STAGE"
cp "$BIN" "$STAGE/$EXE"
case "$TARGET" in
  windows-*) cp "$ROOT/start.bat" "$STAGE/start.bat" ;;
  *) cp "$ROOT/start.sh" "$STAGE/start.sh" ;;
esac
cat > "$STAGE/README.txt" <<EOS
BlockVerse $VERSION ($TARGET)
Run the server with: $( [ "$EXE" = "blockverse.exe" ] && echo start.bat || echo ./start.sh )
Settings: server.properties (created on the first start, or by: $EXE --init)
Logs: logs/latest.log   Plugins: plugins/
BlockVerse is not an official Minecraft product and is not approved by or associated with Mojang.
EOS
for file in LICENSE LICENSE-APACHE-2.0 NOTICE PLUGIN-EXCEPTION.md; do cp "$ROOT/$file" "$STAGE/$file"; done
chmod +x "$STAGE/$EXE"
[ ! -f "$STAGE/start.sh" ] || chmod +x "$STAGE/start.sh"

# Archive
( cd "$OUT" && tar -czf "$NAME.tar.gz" "$NAME" )
if command -v sha256sum >/dev/null 2>&1; then
  ( cd "$OUT" && sha256sum "$NAME.tar.gz" > "$NAME.tar.gz.sha256" )
else
  ( cd "$OUT" && shasum -a 256 "$NAME.tar.gz" > "$NAME.tar.gz.sha256" )
fi
rm -rf "$STAGE"
echo "Created $OUT/$NAME.tar.gz and $OUT/$NAME.tar.gz.sha256"
