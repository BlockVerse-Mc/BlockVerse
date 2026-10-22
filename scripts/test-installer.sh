#!/bin/sh
set -eu

# Paths
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
FAKE_HOME="$WORK/home"
REMOTE="$WORK/remote"
RELEASES="$WORK/releases"
BIN_DIR="$WORK/bin"
mkdir -p "$FAKE_HOME" "$REMOTE" "$RELEASES" "$BIN_DIR"

# Helpers
fails=0
check() {
  if eval "$2"; then
    echo "ok: $1"
  else
    echo "FAILED: $1"
    fails=$((fails + 1))
  fi
}

# Fake toolchain
cat > "$BIN_DIR/rustc" <<'EOS'
#!/bin/sh
echo "rustc 1.90.0 (fake)"
EOS
cat > "$BIN_DIR/cargo" <<'EOS'
#!/bin/sh
manifest=""
while [ $# -gt 0 ]; do
  if [ "$1" = "--manifest-path" ]; then manifest="$2"; fi
  shift
done
dir="$(dirname "$manifest")"
mkdir -p "$dir/target/release"
cat > "$dir/target/release/blockverse" <<'EOB'
#!/bin/sh
case "${1:-}" in
  --version) echo "BlockVerse fake"; exit 0 ;;
  --init) echo "init $*" > init.txt; touch server.properties; exit 0 ;;
esac
exit 0
EOB
chmod +x "$dir/target/release/blockverse"
EOS
chmod +x "$BIN_DIR/rustc" "$BIN_DIR/cargo"

# Fake repository
for f in start.sh start.bat LICENSE LICENSE-APACHE-2.0 NOTICE PLUGIN-EXCEPTION.md; do cp "$ROOT/$f" "$REMOTE/$f"; done
mkdir -p "$REMOTE/scripts" "$REMOTE/crates/bv-server"
cp "$ROOT/scripts/package-release.sh" "$REMOTE/scripts/"
printf '[package]\nname = "bv-server"\nversion = "1.0.0"\n' > "$REMOTE/crates/bv-server/Cargo.toml"
(
  cd "$REMOTE"
  git init -q -b main
  git config user.email test@example.com
  git config user.name test
  git add -A
  git commit -q -m init
  git tag v1.0.0
  echo extra > extra.txt
  git add -A
  git commit -q -m second
)

# Target name
case "$(uname -m)" in
  aarch64|arm64) ARCH=aarch64 ;;
  *) ARCH=x86_64 ;;
esac
TARGET="linux-$ARCH"

# Fake release
printf '#!/bin/sh\ncase "${1:-}" in --version) echo "BlockVerse prebuilt"; exit 0 ;; --init) touch server.properties; exit 0 ;; esac\n' > "$WORK/prebuilt"
chmod +x "$WORK/prebuilt"
(cd "$REMOTE" && sh scripts/package-release.sh "$WORK/prebuilt" "$TARGET" >/dev/null)
mkdir -p "$RELEASES/v1.0.0"
cp "$REMOTE"/dist/blockverse-1.0.0-"$TARGET".tar.gz* "$RELEASES/v1.0.0/"
rm -rf "$REMOTE/dist"

# Environment
export HOME="$FAKE_HOME"
export PATH="$BIN_DIR:$PATH"
export BV_REPO_URL="file://$REMOTE"
export BV_RELEASE_BASE="file://$RELEASES"
export BV_SERVER_NAME="Test Server"
export BV_LANGUAGE="fa"
INSTALL="$ROOT/install.sh"

# Source build of main
export BV_DIR="$WORK/main"
out="$(sh "$INSTALL" 2>&1)"
check "main prints only the progress message and the result" '[ "$(printf "%s\n" "$out" | wc -l)" -eq 2 ]'
check "main shows Preparing BlockVerse" 'printf "%s\n" "$out" | head -n 1 | grep -qx "Preparing BlockVerse"'
check "main installs the binary" '[ -x "$BV_DIR/blockverse" ]'
check "main installs start.sh" '[ -x "$BV_DIR/start.sh" ]'
check "main was built from the newest commit" '[ -f "$BV_DIR/source/extra.txt" ]'
check "main creates server.properties in bvmc" '[ -f "$BV_DIR/bvmc/server.properties" ]'
check "init received the answers" 'grep -q "server-name Test Server --language fa" "$BV_DIR/bvmc/init.txt"'
check "VERSION file says main" '[ "$(cat "$BV_DIR/VERSION")" = "main" ]'

# Prebuilt tag
export BV_DIR="$WORK/tag"
out="$(sh "$INSTALL" v1.0.0 2>&1)"
check "tag installs the prebuilt file" 'grep -q prebuilt "$BV_DIR/blockverse"'
check "tag did not build from source" '[ ! -d "$BV_DIR/source" ]'
check "tag installs start.sh" '[ -x "$BV_DIR/start.sh" ]'
check "tag installs the licenses" '[ -f "$BV_DIR/LICENSE" ] && [ -f "$BV_DIR/NOTICE" ]'
check "tag runs init in bvmc" '[ -f "$BV_DIR/bvmc/server.properties" ]'
check "VERSION file says v1.0.0" '[ "$(cat "$BV_DIR/VERSION")" = "v1.0.0" ]'

# Same command through a pipe
export BV_DIR="$WORK/piped"
out="$(cat "$INSTALL" | sh -s -- v1.0.0 2>&1)"
check "piped install with a version works" '[ -x "$BV_DIR/start.sh" ]'

# Tag without release files falls back to source
export BV_DIR="$WORK/fallback"
mv "$RELEASES/v1.0.0" "$RELEASES/hidden"
out="$(sh "$INSTALL" v1.0.0 2>&1)"
check "missing release file falls back to the source at the tag" '[ -d "$BV_DIR/source" ] && [ ! -f "$BV_DIR/source/extra.txt" ]'
mv "$RELEASES/hidden" "$RELEASES/v1.0.0"

# Damaged download is refused
export BV_DIR="$WORK/damaged"
echo "0000" > "$RELEASES/v1.0.0/blockverse-1.0.0-$TARGET.tar.gz.sha256"
if out="$(sh "$INSTALL" v1.0.0 2>&1)"; then status=0; else status=1; fi
check "a checksum mismatch stops the install" '[ "$status" -eq 1 ] && printf "%s\n" "$out" | grep -q "checksum mismatch"'
check "nothing is installed after a mismatch" '[ ! -f "$BV_DIR/blockverse" ]'

# Invalid version
export BV_DIR="$WORK/invalid"
if out="$(sh "$INSTALL" '--bad;x' 2>&1)"; then status=0; else status=1; fi
check "an invalid version is refused" '[ "$status" -eq 1 ] && printf "%s\n" "$out" | grep -q "invalid version"'

# Start script
export BV_DIR="$WORK/main"
check "start.sh runs the installed binary in bvmc" '(cd "$WORK" && sh "$BV_DIR/start.sh" --version | grep -q "BlockVerse fake")'

if [ "$fails" -ne 0 ]; then
  echo "INSTALLER TESTS FAILED: $fails"
  exit 1
fi
echo "INSTALLER TESTS PASSED"
