#!/bin/sh
set -eu

VERSION="${1:-main}"
SLUG="${BV_REPO_SLUG:-BlockVerse-Mc/BlockVerse}"
REPO_URL="${BV_REPO_URL:-https://github.com/$SLUG.git}"
RELEASE_BASE="${BV_RELEASE_BASE:-https://github.com/$SLUG/releases/download}"
DIR="${BV_DIR:-$HOME/blockverse-server}"
FROM_SOURCE="${BV_FROM_SOURCE:-0}"
SRC="$DIR/source"
LOG="$DIR/install.log"

mkdir -p "$DIR"
: > "$LOG"

fail() {
    printf 'BlockVerse install failed: %s\n' "$1" >&2
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

can_ask() {
    ( exec < /dev/tty ) 2>/dev/null
}

ask() {
    answer=""
    if can_ask; then
        printf '%s [%s]: ' "$1" "$2" > /dev/tty
        read -r answer < /dev/tty || answer=""
    fi
    if [ -n "$answer" ]; then
        printf '%s' "$answer"
    else
        printf '%s' "$2"
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

install_prebuilt() {
    file="blockverse-${VERSION#v}-$TARGET.tar.gz"
    tmp=$(mktemp -d)
    if ! fetch "$RELEASE_BASE/$VERSION/$file" "$tmp/server.tar.gz"; then
        rm -rf "$tmp"
        return 1
    fi
    if ! fetch "$RELEASE_BASE/$VERSION/$file.sha256" "$tmp/server.sha256"; then
        rm -rf "$tmp"
        return 1
    fi
    want=$(cut -d ' ' -f 1 "$tmp/server.sha256" | head -n 1)
    got=$(sha256_of "$tmp/server.tar.gz") || got=""
    if [ -z "$want" ] || [ "$want" != "$got" ]; then
        rm -rf "$tmp"
        fail "checksum mismatch, the download is damaged or was modified"
    fi
    mkdir -p "$tmp/out"
    if ! tar -xzf "$tmp/server.tar.gz" -C "$tmp/out" --strip-components=1 >>"$LOG" 2>&1; then
        rm -rf "$tmp"
        return 1
    fi
    chmod +x "$tmp/out/blockverse"
    if ! "$tmp/out/blockverse" --version >>"$LOG" 2>&1; then
        rm -rf "$tmp"
        return 1
    fi
    rm -f "$DIR/blockverse" "$DIR/start.sh"
    cp -R "$tmp/out/." "$DIR/"
    rm -rf "$tmp"
    return 0
}

build_from_source() {
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

    mkdir -p "$SRC"
    cd "$SRC"
    [ -d .git ] || run git init -q
    git remote remove origin >/dev/null 2>&1 || true
    run git remote add origin "$REPO_URL"
    run git fetch -q --depth 1 origin "$VERSION"
    run git checkout -q -f FETCH_HEAD

    if is_termux; then
        run env CARGO_PROFILE_RELEASE_LTO=false CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16 CARGO_PROFILE_RELEASE_DEBUG=0 \
            cargo build --release -j "${BV_JOBS:-2}" --manifest-path crates/bv-server/Cargo.toml
    else
        run cargo build --release --manifest-path crates/bv-server/Cargo.toml
    fi

    run rm -f "$DIR/blockverse" "$DIR/start.sh"
    run cp crates/bv-server/target/release/blockverse "$DIR/blockverse"
    run cp start.sh "$DIR/start.sh"
    for extra in LICENSE LICENSE-APACHE-2.0 NOTICE PLUGIN-EXCEPTION.md; do
        [ -f "$extra" ] && cp "$extra" "$DIR/$extra"
    done
    cd "$DIR"
}

case "$VERSION" in
    -*|*[!A-Za-z0-9._/-]*) fail "invalid version" ;;
esac

printf 'Preparing BlockVerse\n'

export GIT_TERMINAL_PROMPT=0
if [ -f "$HOME/.cargo/env" ]; then
    . "$HOME/.cargo/env"
fi

TARGET=$(detect_target)
[ -n "$TARGET" ] || fail "unsupported system"

NAME="${BV_SERVER_NAME:-}"
[ -n "$NAME" ] || NAME=$(ask 'Server name' 'BlockVerse')
LANG_CODE="${BV_LANGUAGE:-}"
[ -n "$LANG_CODE" ] || LANG_CODE=$(ask 'Language (en or fa)' 'en')
case "$LANG_CODE" in
    en|fa) ;;
    *) LANG_CODE=en ;;
esac

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
mkdir -p "$DIR/bvmc"
( cd "$DIR/bvmc" && run "$DIR/blockverse" --init --server-name "$NAME" --language "$LANG_CODE" )
printf '%s\n' "$VERSION" > "$DIR/VERSION"

printf 'BlockVerse ready: %s/start.sh\n' "$DIR"
