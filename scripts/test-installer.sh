#!/bin/sh
set -eu

# Paths
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
FAKE_HOME="$WORK/home"
REMOTE="$WORK/remote"
RELEASES="$WORK/releases"
API="$WORK/api"
BIN_DIR="$WORK/bin"
RUNS="$WORK/runs"
TMP_AREA="$WORK/tmp"
mkdir -p "$FAKE_HOME" "$REMOTE" "$RELEASES" "$API/repos/BlockVerse-Mc/BlockVerse/releases" "$BIN_DIR" "$RUNS" "$TMP_AREA"

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

has_all() {
  text="$1"
  shift
  for needle in "$@"; do
    printf '%s\n' "$text" | grep -q "$needle" || return 1
  done
}

newdir() {
  mkdir -p "$RUNS/$1"
  printf '%s' "$RUNS/$1"
}

install_in() {
  name="$1"
  shift
  ( cd "$RUNS/$name" && sh "$INSTALL" "$@" 2>&1 )
}

no_folders() {
  [ -z "$(find "$1" -mindepth 1 -type d)" ]
}

tmp_clean() {
  [ -z "$(ls -A "$TMP_AREA")" ]
}

# Fake toolchain
cat > "$BIN_DIR/rustc" <<'EOS'
#!/bin/sh
echo "rustc 1.90.0 (fake)"
EOS
cat > "$BIN_DIR/cargo" <<'EOS'
#!/bin/sh
case "${1:-}" in
  tree)
    printf 'blockverse v1.0.0\nalpha v0.1.0\nbeta v0.2.0\n[build-dependencies]\nalpha v0.1.0 (*)\n'
    exit 0
    ;;
  fetch)
    exit 0
    ;;
esac
manifest=""
while [ $# -gt 0 ]; do
  if [ "$1" = "--manifest-path" ]; then manifest="$2"; fi
  shift
done
printf '   Compiling alpha v0.1.0\n   Compiling beta v0.2.0\n'
sleep 2
printf '   Compiling blockverse v1.0.0\n'
if [ -n "${FAKE_BUILD_FAIL:-}" ]; then
  echo "error[E0000]: fake failure" >&2
  exit 101
fi
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

# Fake releases
make_release() {
  marker="$1"
  label="$2"
  folder="$3"
  printf '#!/bin/sh\ncase "${1:-}" in --version) echo "%s"; exit 0 ;; --init) echo "init $*" > init.txt; touch server.properties; exit 0 ;; esac\n' "$marker" > "$WORK/prebuilt"
  chmod +x "$WORK/prebuilt"
  (cd "$REMOTE" && sh scripts/package-release.sh "$WORK/prebuilt" "$TARGET" $label >/dev/null)
  mkdir -p "$RELEASES/$folder"
  cp "$REMOTE"/dist/blockverse-*-"$TARGET".tar.gz* "$RELEASES/$folder/"
  rm -rf "$REMOTE/dist"
}
make_release "prebuilt-release" "" "v1.0.0"
make_release "prebuilt-nightly" "nightly" "nightly"
printf '{"tag_name": "v1.0.0", "prerelease": false}\n' > "$API/repos/BlockVerse-Mc/BlockVerse/releases/latest"

# Environment
export HOME="$FAKE_HOME"
export PATH="$BIN_DIR:$PATH"
export TMPDIR="$TMP_AREA"
export BV_REPO_URL="file://$REMOTE"
export BV_RELEASE_BASE="file://$RELEASES"
export BV_API_BASE="file://$API"
export BV_POLL=1
unset BV_DIR BV_SERVER_NAME BV_LANGUAGE NO_COLOR BV_FORCE BV_FROM_SOURCE || true
INSTALL="$ROOT/install.sh"

# Newest release
dir=$(newdir latest)
out="$(install_in latest)"
check "latest starts with the progress title" 'printf "%s\n" "$out" | head -n 1 | grep -qx "Preparing BlockVerse"'
check "latest installs the newest release without compiling" 'has_all "$out" "Downloading BlockVerse v1.0.0" "Verifying download" "Installing files" && ! printf "%s\n" "$out" | grep -q "Compiling"'
check "latest installed the release file" 'grep -q prebuilt-release "$dir/blockverse" && [ "$(cat "$dir/VERSION")" = "v1.0.0" ]'
check "latest prints Done and how to start" '[ "$(printf "%s\n" "$out" | tail -n 2 | head -n 1)" = "Done" ] && printf "%s\n" "$out" | tail -n 1 | grep -q "./start.sh"'
check "latest installs start.sh and the licenses" '[ -x "$dir/start.sh" ] && [ -f "$dir/LICENSE" ] && [ -f "$dir/NOTICE" ]'
check "latest creates no folder" 'no_folders "$dir"'
check "latest leaves no temporary files" 'tmp_clean'

# Update in the same folder
out="$(install_in latest)"
check "running the command again in the same folder updates it" '[ -x "$dir/blockverse" ] && printf "%s\n" "$out" | tail -n 2 | head -n 1 | grep -qx "Done"'

# No release yet: the rolling build
mv "$API/repos/BlockVerse-Mc/BlockVerse/releases/latest" "$WORK/latest.hidden"
dir=$(newdir nightly-fallback)
out="$(install_in nightly-fallback)"
check "without a release the newest rolling build is installed" 'grep -q prebuilt-nightly "$dir/blockverse" && [ "$(cat "$dir/VERSION")" = "nightly" ]'
check "the rolling build needs no compiler" '! printf "%s\n" "$out" | grep -q "Compiling" && no_folders "$dir" && tmp_clean'
mv "$WORK/latest.hidden" "$API/repos/BlockVerse-Mc/BlockVerse/releases/latest"

# Named channels
dir=$(newdir nightly)
out="$(install_in nightly nightly)"
check "nightly installs the rolling build" 'grep -q prebuilt-nightly "$dir/blockverse"'
dir=$(newdir tag)
out="$(install_in tag v1.0.0)"
check "a version tag installs that release" 'grep -q prebuilt-release "$dir/blockverse" && [ "$(cat "$dir/VERSION")" = "v1.0.0" ] && no_folders "$dir"'

# Same command through a pipe
dir=$(newdir piped)
out="$(cd "$dir" && cat "$INSTALL" | sh -s -- v1.0.0 2>&1)"
check "piped install with a version works" '[ -x "$dir/start.sh" ] && no_folders "$dir"'

# Source build
dir=$(newdir main)
out="$(install_in main main)"
check "main builds from source and shows every stage" 'has_all "$out" "Checking system" "Installing prerequisites" "Downloading source" "Downloading dependencies" "Compiling BlockVerse" "Installing files"'
check "main shows a percentage while compiling" 'printf "%s\n" "$out" | grep -qE "Compiling BlockVerse \[[#.]*\] %[0-9]+$"'
check "main ends the compile step with 100 percent and done" 'printf "%s\n" "$out" | grep -q "Compiling BlockVerse \[[#.]*] %100 done"'
check "main installs the built binary in the folder" 'grep -q "BlockVerse fake" "$dir/blockverse" && [ -x "$dir/start.sh" ] && [ "$(cat "$dir/VERSION")" = "main" ]'
check "main was built from the newest commit" '! [ -e "$dir/extra.txt" ]'
check "main leaves no source folder, no folder and no temporary files" 'no_folders "$dir" && tmp_clean'

# Tag without release files falls back to source
mv "$RELEASES/v1.0.0" "$RELEASES/hidden"
dir=$(newdir fallback)
out="$(install_in fallback v1.0.0)"
check "a missing release file falls back to the source at the tag" 'has_all "$out" "no prebuilt file, building from source" "Compiling BlockVerse" && grep -q "BlockVerse fake" "$dir/blockverse"'
check "the fallback leaves no folder and no temporary files" 'no_folders "$dir" && tmp_clean'
mv "$RELEASES/hidden" "$RELEASES/v1.0.0"

# Answers given through variables
dir=$(newdir answers)
out="$(cd "$dir" && BV_SERVER_NAME="Test Server" BV_LANGUAGE=de sh "$INSTALL" v1.0.0 2>&1)"
check "answers create server.properties in the folder" '[ -f "$dir/server.properties" ] && grep -q "server-name Test Server --language de" "$dir/init.txt"'

# Damaged download is refused
dir=$(newdir damaged)
cp "$RELEASES/v1.0.0/blockverse-1.0.0-$TARGET.tar.gz.sha256" "$WORK/sum.backup"
echo "0000" > "$RELEASES/v1.0.0/blockverse-1.0.0-$TARGET.tar.gz.sha256"
if out="$(install_in damaged v1.0.0)"; then status=0; else status=1; fi
check "a checksum mismatch stops the install" '[ "$status" -eq 1 ] && printf "%s\n" "$out" | grep -q "checksum mismatch"'
check "nothing is installed after a mismatch" '[ ! -e "$dir/blockverse" ] && tmp_clean'
cp "$WORK/sum.backup" "$RELEASES/v1.0.0/blockverse-1.0.0-$TARGET.tar.gz.sha256"

# Failing build
dir=$(newdir buildfail)
if out="$(cd "$dir" && FAKE_BUILD_FAIL=1 sh "$INSTALL" main 2>&1)"; then status=0; else status=1; fi
check "a failing build stops the install and shows the compiler output" '[ "$status" -eq 1 ] && has_all "$out" "BlockVerse install failed" "fake failure"'
check "a failing build leaves only the log in the folder" '[ ! -e "$dir/blockverse" ] && [ -f "$dir/install.log" ] && no_folders "$dir" && tmp_clean'

# Folder rules
dir=$(newdir busy)
echo "mine" > "$dir/notes.txt"
if out="$(install_in busy v1.0.0)"; then status=0; else status=1; fi
check "a folder with other files is refused" '[ "$status" -eq 1 ] && printf "%s\n" "$out" | grep -q "not empty" && [ ! -e "$dir/blockverse" ] && [ "$(cat "$dir/notes.txt")" = mine ]'
if out="$(cd "$HOME" && sh "$INSTALL" v1.0.0 2>&1)"; then status=0; else status=1; fi
check "the home folder is refused" '[ "$status" -eq 1 ] && printf "%s\n" "$out" | grep -q "empty folder you created" && [ ! -e "$HOME/blockverse" ]'

# Invalid version
dir=$(newdir invalid)
if out="$(install_in invalid '--bad;x')"; then status=0; else status=1; fi
check "an invalid version is refused" '[ "$status" -eq 1 ] && printf "%s\n" "$out" | grep -q "invalid version"'

# Start script
dir="$RUNS/main"
check "start.sh runs the installed binary in the same folder" '(cd "$WORK" && sh "$dir/start.sh" --version | grep -q "BlockVerse fake") && no_folders "$dir"'

if [ "$fails" -ne 0 ]; then
  echo "INSTALLER TESTS FAILED: $fails"
  exit 1
fi
echo "INSTALLER TESTS PASSED"
