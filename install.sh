#!/bin/sh
set -eu

VERSION="${1:-main}"
SLUG="${BV_REPO_SLUG:-BlockVerse-Mc/BlockVerse}"
REPO_URL="${BV_REPO_URL:-https://github.com/$SLUG.git}"
RELEASE_BASE="${BV_RELEASE_BASE:-https://github.com/$SLUG/releases/download}"
DIR="${BV_DIR:-$HOME/blockverse-server}"
FROM_SOURCE="${BV_FROM_SOURCE:-0}"
POLL="${BV_POLL:-1}"
SRC="$DIR/source"
LOG="$DIR/install.log"
BUILD_LOG="$DIR/build.log"
MANIFEST="crates/bv-server/Cargo.toml"

mkdir -p "$DIR"
: > "$LOG"

TTY=0
C_RESET=""
C_GREEN=""
C_RED=""
C_CYAN=""
C_YELLOW=""
C_BOLD=""
if [ -t 1 ] && [ -z "${NO_COLOR:-}" ] && [ "${TERM:-dumb}" != "dumb" ]; then
    TTY=1
    C_RESET=$(printf '\033[0m')
    C_GREEN=$(printf '\033[32m')
    C_RED=$(printf '\033[31m')
    C_CYAN=$(printf '\033[36m')
    C_YELLOW=$(printf '\033[33m')
    C_BOLD=$(printf '\033[1m')
fi

STEP_OPEN=0
LAST_BUCKET=-1

begin() {
    STEP_OPEN=1
    printf '  %s-%s %s ... ' "$C_CYAN" "$C_RESET" "$1"
}

finish() {
    STEP_OPEN=0
    printf '%sdone%s\n' "$C_GREEN" "$C_RESET"
}

fail() {
    if [ "$STEP_OPEN" = 1 ]; then
        printf '%sfailed%s\n' "$C_RED" "$C_RESET" >&2
    fi
    printf '%sBlockVerse install failed:%s %s\n' "$C_RED" "$C_RESET" "$1" >&2
    printf 'Log: %s\n' "$LOG" >&2
    exit 1
}

run() {
    "$@" >>"$LOG" 2>&1 || fail "$*"
}

have() {
    command -v "$1" >/dev/null 2>&1
}

is_termux() {
    case "${PREFIX:-}" in
        *com.termux*) return 0 ;;
    esac
    return 1
}

fetch() {
    if have curl; then
        curl -fsSL -o "$2" "$1" >>"$LOG" 2>&1
    elif have wget; then
        wget -q -O "$2" "$1" >>"$LOG" 2>&1
    else
        return 1
    fi
}

sha256_of() {
    if have sha256sum; then
        sha256sum "$1" | cut -d ' ' -f 1
    elif have shasum; then
        shasum -a 256 "$1" | cut -d ' ' -f 1
    else
        return 1
    fi
}

rust_ok() {
    version=$(rustc --version 2>/dev/null | cut -d ' ' -f 2)
    [ -n "$version" ] || return 1
    major=${version%%.*}
    rest=${version#*.}
    minor=${rest%%.*}
    [ "$major" -gt 1 ] 2>/dev/null && return 0
    [ "$major" -eq 1 ] 2>/dev/null && [ "$minor" -ge 85 ] 2>/dev/null
}

detect_target() {
    arch=$(uname -m)
    case "$arch" in
        aarch64|arm64) arch=aarch64 ;;
        x86_64|amd64) arch=x86_64 ;;
        *) return 0 ;;
    esac
    case "$(uname -s)" in
        Linux)
            if is_termux; then
                printf 'android-%s' "$arch"
            else
                printf 'linux-%s' "$arch"
            fi
            ;;
        Darwin) printf 'macos-%s' "$arch" ;;
    esac
}

bar_text() {
    width=24
    filled=$(($1 * width / 100))
    text=""
    i=0
    while [ "$i" -lt "$width" ]; do
        if [ "$i" -lt "$filled" ]; then
            text="$text#"
        else
            text="$text."
        fi
        i=$((i + 1))
    done
    printf '%s' "$text"
}

show_progress() {
    count=$1
    total=$2
    if [ "$total" -gt 0 ]; then
        percent=$((count * 100 / total))
        if [ "$percent" -gt 99 ]; then
            percent=99
        fi
        bar=$(bar_text "$percent")
        if [ "$TTY" = 1 ]; then
            printf '\r\033[K  %s-%s Compiling BlockVerse %s[%s]%s %%%s' "$C_CYAN" "$C_RESET" "$C_GREEN" "$bar" "$C_RESET" "$percent"
        else
            bucket=$((percent / 10))
            if [ "$bucket" != "$LAST_BUCKET" ]; then
                printf '  - Compiling BlockVerse [%s] %%%s\n' "$bar" "$percent"
                LAST_BUCKET=$bucket
            fi
        fi
    elif [ "$TTY" = 1 ]; then
        printf '\r\033[K  %s-%s Compiling BlockVerse ... %s crates' "$C_CYAN" "$C_RESET" "$count"
    fi
}

show_compiled() {
    bar=$(bar_text 100)
    if [ "$TTY" = 1 ]; then
        printf '\r\033[K  %s-%s Compiling BlockVerse %s[%s]%s %%100 %sdone%s\n' "$C_CYAN" "$C_RESET" "$C_GREEN" "$bar" "$C_RESET" "$C_GREEN" "$C_RESET"
    else
        printf '  - Compiling BlockVerse [%s] %%100 done\n' "$bar"
    fi
}

count_units() {
    cargo tree --manifest-path "$1" -e normal,build --prefix none 2>/dev/null | grep -v '^\[' | sed 's/ (\*)$//' | sort -u | wc -l | tr -d ' '
}

compile() {
    total=$(count_units "$MANIFEST")
    case "$total" in
        ''|*[!0-9]*) total=0 ;;
    esac
    : > "$BUILD_LOG"
    if is_termux; then
        env CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 CARGO_PROFILE_RELEASE_DEBUG=0 \
            cargo build --release --color never -j "${BV_JOBS:-2}" --manifest-path "$MANIFEST" >"$BUILD_LOG" 2>&1 &
    else
        cargo build --release --color never --manifest-path "$MANIFEST" >"$BUILD_LOG" 2>&1 &
    fi
    pid=$!
    trap 'kill "$pid" 2>/dev/null; exit 130' INT TERM
    LAST_BUCKET=-1
    while kill -0 "$pid" 2>/dev/null; do
        compiled=$(grep -c '^ *Compiling ' "$BUILD_LOG" 2>/dev/null || true)
        show_progress "$compiled" "$total"
        sleep "$POLL"
    done
    if wait "$pid"; then
        status=0
    else
        status=$?
    fi
    trap - INT TERM
    cat "$BUILD_LOG" >> "$LOG"
    if [ "$status" -ne 0 ]; then
        if [ "$TTY" = 1 ]; then
            printf '\r\033[K'
        fi
        printf '  - Compiling BlockVerse %sfailed%s\n' "$C_RED" "$C_RESET" >&2
        tail -n 12 "$BUILD_LOG" >&2
        fail "the build failed"
    fi
    show_compiled
}

install_prebuilt() {
    file="blockverse-${VERSION#v}-$TARGET.tar.gz"
    tmp=$(mktemp -d)
    begin "Downloading BlockVerse $VERSION"
    if ! fetch "$RELEASE_BASE/$VERSION/$file" "$tmp/server.tar.gz" || ! fetch "$RELEASE_BASE/$VERSION/$file.sha256" "$tmp/server.sha256"; then
        STEP_OPEN=0
        printf '%sno prebuilt file, building from source%s\n' "$C_YELLOW" "$C_RESET"
        rm -rf "$tmp"
        return 1
    fi
    finish
    begin "Verifying download"
    want=$(cut -d ' ' -f 1 "$tmp/server.sha256" | head -n 1)
    got=$(sha256_of "$tmp/server.tar.gz") || got=""
    if [ -z "$want" ] || [ "$want" != "$got" ]; then
        rm -rf "$tmp"
        fail "checksum mismatch, the download is damaged or was modified"
    fi
    finish
    begin "Installing files"
    mkdir -p "$tmp/out"
    if ! tar -xzf "$tmp/server.tar.gz" -C "$tmp/out" --strip-components=1 >>"$LOG" 2>&1; then
        STEP_OPEN=0
        printf '%sunpack failed, building from source%s\n' "$C_YELLOW" "$C_RESET"
        rm -rf "$tmp"
        return 1
    fi
    chmod +x "$tmp/out/blockverse"
    if ! "$tmp/out/blockverse" --version >>"$LOG" 2>&1; then
        STEP_OPEN=0
        printf '%sthe file does not run here, building from source%s\n' "$C_YELLOW" "$C_RESET"
        rm -rf "$tmp"
        return 1
    fi
    rm -f "$DIR/blockverse" "$DIR/start.sh"
    cp -R "$tmp/out/." "$DIR/"
    rm -rf "$tmp"
    finish
    return 0
}

build_from_source() {
    begin "Installing prerequisites"
    if is_termux; then
        have git || run pkg install -y git
        rust_ok || run pkg install -y rust
    else
        have git || fail "git is required"
        have cc || have gcc || have clang || fail "a C compiler is required"
        if ! rust_ok; then
            have curl || fail "curl is required"
            curl -sSf https://sh.rustup.rs -o "$DIR/rustup-init.sh" >>"$LOG" 2>&1 || fail "rustup download"
            run env RUSTUP_INIT_SKIP_PATH_CHECK=yes sh "$DIR/rustup-init.sh" -y --profile minimal
            . "$HOME/.cargo/env"
        fi
    fi
    rust_ok || fail "Rust 1.85 or newer is required"
    finish

    begin "Downloading source"
    mkdir -p "$SRC"
    cd "$SRC"
    [ -d .git ] || run git init -q
    git remote remove origin >/dev/null 2>&1 || true
    run git remote add origin "$REPO_URL"
    run git fetch -q --depth 1 origin "$VERSION"
    run git checkout -q -f FETCH_HEAD
    finish

    begin "Downloading dependencies"
    host=$(rustc -vV 2>/dev/null | sed -n 's/^host: //p')
    if [ -n "$host" ]; then
        run cargo fetch --manifest-path "$MANIFEST" --target "$host"
    else
        run cargo fetch --manifest-path "$MANIFEST"
    fi
    finish

    compile

    begin "Installing files"
    run rm -f "$DIR/blockverse" "$DIR/start.sh"
    run cp crates/bv-server/target/release/blockverse "$DIR/blockverse"
    run cp start.sh "$DIR/start.sh"
    for extra in LICENSE LICENSE-APACHE-2.0 NOTICE PLUGIN-EXCEPTION.md; do
        [ -f "$extra" ] && cp "$extra" "$DIR/$extra"
    done
    cd "$DIR"
    finish
}

case "$VERSION" in
    -*|*[!A-Za-z0-9._/-]*) fail "invalid version" ;;
esac

printf '%sPreparing BlockVerse%s\n' "$C_BOLD" "$C_RESET"

export GIT_TERMINAL_PROMPT=0
if [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi

begin "Checking system"
TARGET=$(detect_target)
[ -n "$TARGET" ] || fail "unsupported system"
finish

INSTALLED=0
case "$VERSION" in
    v[0-9]*)
        if [ "$FROM_SOURCE" != 1 ]; then
            if install_prebuilt; then
                INSTALLED=1
            fi
        fi
        ;;
esac
if [ "$INSTALLED" != 1 ]; then
    build_from_source
fi

chmod +x "$DIR/blockverse" "$DIR/start.sh"
if [ -n "${BV_SERVER_NAME:-}" ] || [ -n "${BV_LANGUAGE:-}" ]; then
    begin "Creating settings"
    mkdir -p "$DIR/bvmc"
    ( cd "$DIR/bvmc" && run "$DIR/blockverse" --init --server-name "${BV_SERVER_NAME:-BlockVerse}" --language "${BV_LANGUAGE:-en}" )
    finish
fi
printf '%s\n' "$VERSION" > "$DIR/VERSION"

printf '%sDone%s\n' "$C_GREEN$C_BOLD" "$C_RESET"
printf 'Start the server: %s/start.sh\n' "$DIR"
