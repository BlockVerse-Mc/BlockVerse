#!/bin/sh
set -eu

# Paths
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT
export HOME="$WORK/home"
REPO="$HOME/BlockVerse"
DL="$HOME/storage/downloads"
BIN="$WORK/bin"
mkdir -p "$HOME" "$DL" "$BIN"

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

# Repository with a remote
git init -q --bare -b main "$WORK/remote.git"
git init -q -b main "$REPO"
(
  cd "$REPO"
  git config user.email test@example.com
  git config user.name test
  echo base > base.txt
  git add -A
  git commit -q -m base
  git remote add origin "$WORK/remote.git"
  git push -q -u origin main
)

# Update files, the highest number must win
mkdir -p "$WORK/u2" "$WORK/u10"
echo two > "$WORK/u2/two.txt"
echo ten > "$WORK/u10/ten.txt"
(cd "$WORK/u2" && zip -q "$DL/BlockVerse-Update-2.zip" two.txt)
(cd "$WORK/u10" && zip -q "$DL/BlockVerse-Update-10.zip" ten.txt)

# Fake gh
cat > "$BIN/gh" <<'EOS'
#!/bin/sh
case "$*" in
  *databaseId*) echo 4242 ;;
  *"run view"*) echo "error[E0000]: fake compile error" ;;
  *) cat "$GH_RUNS" ;;
esac
EOS
chmod +x "$BIN/gh"
export PATH="$BIN:$PATH"
export GH_RUNS="$WORK/runs.txt"
export BV_REPO_DIR="$REPO"
export BV_DOWNLOADS="$DL"
export BV_POLL_SECONDS=1
export BV_WAIT_SECONDS=6
TOOL="$ROOT/scripts/bvdev.sh"

# Apply
printf 'CI completed success\nNightly completed success\n' > "$GH_RUNS"
out="$(sh "$TOOL" apply 2>&1)"
check "apply uses the highest numbered zip" '[ -f "$REPO/ten.txt" ] && [ ! -f "$REPO/two.txt" ]'
check "apply commits and pushes" '[ "$(git -C "$REPO" rev-parse HEAD)" = "$(git -C "$WORK/remote.git" rev-parse main)" ]'
check "apply reports passing builds" 'printf "%s\n" "$out" | grep -q "All builds passed"'
out="$(sh "$TOOL" apply 2>&1)"
check "applying the same update again changes nothing" 'printf "%s\n" "$out" | grep -q "Nothing changed"'

# A failing build is shown
printf 'CI completed failure\nNightly completed success\n' > "$GH_RUNS"
if out="$(sh "$TOOL" apply 2>&1)"; then status=0; else status=1; fi
check "a failed build stops with its log" '[ "$status" -eq 1 ] && printf "%s\n" "$out" | grep -q "fake compile error" && printf "%s\n" "$out" | grep -q "A build failed"'

# Folder safety
if out="$(BV_SERVER_DIR=/tmp/bvdev-outside sh "$TOOL" fresh 2>&1)"; then status=0; else status=1; fi
check "fresh refuses a folder outside the home folder" '[ "$status" -eq 1 ] && printf "%s\n" "$out" | grep -q "refusing"'
if out="$(BV_SERVER_DIR="$HOME" sh "$TOOL" fresh 2>&1)"; then status=0; else status=1; fi
check "fresh refuses the home folder" '[ "$status" -eq 1 ] && printf "%s\n" "$out" | grep -q "refusing"'

# Probe with a fake installer
cat > "$WORK/fake-install.sh" <<'EOS'
#!/bin/sh
printf '#!/bin/sh\nmkdir -p logs\n[ -n "${BV_PROBE:-}" ] && echo "PROBE RESULT step 1/6 dropping [nothing]: REJECTED by the client" > logs/latest.log\nexit 0\n' > start.sh
chmod +x start.sh
echo "log-level=info" > server.properties
EOS
mkdir -p "$HOME/bvmc-sv"
echo old > "$HOME/bvmc-sv/old.txt"
out="$(BV_INSTALL_URL="file://$WORK/fake-install.sh" sh "$TOOL" probe 2>&1)"
check "probe starts from an empty folder" '[ ! -f "$HOME/bvmc-sv/old.txt" ]'
check "probe turns on the debug log" 'grep -q "^log-level=debug" "$HOME/bvmc-sv/server.properties"'
check "probe saves the result lines for sending" 'grep -q "PROBE RESULT" "$DL/bv-probe.txt" && printf "%s\n" "$out" | grep -q "Send those lines"'

# Setup
sh "$TOOL" setup >/dev/null 2>&1
check "setup installs a bvdev command" '[ -x "$HOME/.local/bin/bvdev" ]'

if [ "$fails" -ne 0 ]; then
  echo "BVDEV TESTS FAILED: $fails"
  exit 1
fi
echo "BVDEV TESTS PASSED"
