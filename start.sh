#!/bin/sh
DIR="$(cd "$(dirname "$0")" && pwd)" || exit 1
BIN=""
DATA="$DIR"
if [ -x "$DIR/blockverse" ]; then
    BIN="$DIR/blockverse"
elif [ -x "$DIR/crates/bv-server/target/release/blockverse" ]; then
    BIN="$DIR/crates/bv-server/target/release/blockverse"
    DATA="$DIR/bvmc"
elif [ -x "$DIR/crates/bv-server/target/debug/blockverse" ]; then
    BIN="$DIR/crates/bv-server/target/debug/blockverse"
    DATA="$DIR/bvmc"
fi
if [ -z "$BIN" ]; then
    echo "BlockVerse is not built. Run: cd crates/bv-server && cargo build --release"
    exit 1
fi
if command -v termux-wake-lock >/dev/null 2>&1; then
    termux-wake-lock
fi
mkdir -p "$DATA"
cd "$DATA" || exit 1
exec "$BIN" "$@"
