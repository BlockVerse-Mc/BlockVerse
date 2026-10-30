#!/bin/sh
set -eu

REPO="${BV_REPO_DIR:-$HOME/BlockVerse}"
DOWNLOADS="${BV_DOWNLOADS:-$HOME/storage/downloads}"
SERVER_DIR="${BV_SERVER_DIR:-$HOME/bvmc-sv}"
SLUG="${BV_REPO_SLUG:-BlockVerse-Mc/BlockVerse}"
INSTALL_URL="${BV_INSTALL_URL:-https://raw.githubusercontent.com/$SLUG/main/install.sh}"
WAIT_SECONDS="${BV_WAIT_SECONDS:-1500}"
POLL_SECONDS="${BV_POLL_SECONDS:-20}"

say() {
    printf '%s\n' "$*"
}

die() {
    printf 'bvdev: %s\n' "$*" >&2
    exit 1
}

newest_zip() {
    ls -1 "$DOWNLOADS"/BlockVerse-Update-*.zip 2>/dev/null | sort -V | tail -n 1
}

wait_for_runs() {
    sha=$(git -C "$REPO" rev-parse HEAD)
    waited=0
    say "Waiting for GitHub to build commit $(printf '%s' "$sha" | cut -c1-7)"
    sleep "$POLL_SECONDS"
    while [ "$waited" -le "$WAIT_SECONDS" ]; do
        runs=$(gh run list -R "$SLUG" --commit "$sha" --json name,status,conclusion --jq '.[] | "\(.name) \(.status) \(.conclusion)"' 2>/dev/null || true)
        if [ -n "$runs" ] && ! printf '%s\n' "$runs" | grep -qE ' (queued|in_progress|waiting|pending)( |$)'; then
            printf '%s\n' "$runs"
            if printf '%s\n' "$runs" | grep -qE ' (failure|cancelled|timed_out)( |$)'; then
                return 1
            fi
            return 0
        fi
        printf '.'
        sleep "$POLL_SECONDS"
        waited=$((waited + POLL_SECONDS))
    done
    say ""
    say "Still running after $WAIT_SECONDS s. Check: gh run list -R $SLUG --limit 4"
    return 1
}

show_failure() {
    id=$(gh run list -R "$SLUG" --commit "$(git -C "$REPO" rev-parse HEAD)" --json databaseId,conclusion --jq '[.[] | select(.conclusion == "failure")][0].databaseId' 2>/dev/null || true)
    if [ -n "$id" ] && [ "$id" != "null" ]; then
        say "--- failed log (last lines) ---"
        gh run view "$id" -R "$SLUG" --log-failed 2>/dev/null | grep -v '^$' | tail -n 40 || true
    fi
}

cmd_apply() {
    [ -d "$REPO/.git" ] || die "no repository at $REPO"
    zip=$(newest_zip)
    [ -n "$zip" ] || die "no BlockVerse-Update-*.zip in $DOWNLOADS. Download the update first"
    say "Applying $(basename "$zip")"
    unzip -oq "$zip" -d "$REPO"
    cd "$REPO"
    git add -A
    if git diff --cached --quiet; then
        say "Nothing changed, this update is already applied."
    else
        git commit -q -m "Apply $(basename "$zip" .zip)"
        git push -q
        say "Pushed."
    fi
    if wait_for_runs; then
        say "All builds passed. Next: bvdev probe"
    else
        show_failure
        say "A build failed. Send the lines above."
        exit 1
    fi
}

safe_server_dir() {
    case "$SERVER_DIR" in
        "$HOME"/*) ;;
        *) die "refusing to touch $SERVER_DIR (it must be inside $HOME)" ;;
    esac
    [ "$SERVER_DIR" != "$HOME" ] || die "refusing to touch $HOME"
}

cmd_fresh() {
    safe_server_dir
    rm -rf "$SERVER_DIR"
    mkdir -p "$SERVER_DIR"
    cd "$SERVER_DIR"
    curl -fsSL "$INSTALL_URL" | BV_SERVER_NAME=BlockVerse BV_LANGUAGE=en bash
}

cmd_run() {
    [ -x "$SERVER_DIR/start.sh" ] || die "no server in $SERVER_DIR. Run: bvdev fresh"
    cd "$SERVER_DIR"
    ./start.sh "$@"
}

cmd_probe() {
    cmd_fresh
    cd "$SERVER_DIR"
    sed -i 's/^log-level=.*/log-level=debug/' server.properties
    say ""
    say "Probe: join 6 times in a row. Each time wait 10 s, then leave. Then type: stop"
    BV_PROBE=1 BV_NO_SETUP=1 ./start.sh || true
    out="$DOWNLOADS/bv-probe.txt"
    grep -h "PROBE" logs/latest.log > "$out" 2>/dev/null || true
    say ""
    say "--- results ---"
    cat "$out" 2>/dev/null || true
    say "Saved to $out. Send those lines."
}

cmd_setup() {
    target="${PREFIX:-$HOME/.local}/bin"
    mkdir -p "$target"
    printf '#!/bin/sh\nexec sh "%s/scripts/bvdev.sh" "$@"\n' "$REPO" > "$target/bvdev"
    chmod +x "$target/bvdev"
    say "Installed $target/bvdev. Commands: bvdev apply | fresh | run | probe"
}

case "${1:-help}" in
    apply) cmd_apply ;;
    fresh) cmd_fresh ;;
    run) shift; cmd_run "$@" ;;
    probe) cmd_probe ;;
    setup) cmd_setup ;;
    *)
        say "bvdev apply   apply the newest BlockVerse-Update zip, push it and wait for the builds"
        say "bvdev fresh   empty ~/bvmc-sv and install the newest build there"
        say "bvdev run     start the server in ~/bvmc-sv"
        say "bvdev probe   fresh install, then the automatic packet probe"
        say "bvdev setup   install the bvdev command"
        ;;
esac
