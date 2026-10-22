#!/bin/sh
DIR="$(cd "$(dirname "$0")" && pwd)" || exit 1
BIN=""
for candidate in "$DIR/blockverse" "$DIR/crates/bv-server/target/release/blockverse" "$DIR/crates/bv-server/target/debug/blockverse"; do
    if [ -x "$candidate" ]; then
        BIN="$candidate"
        break
    fi
done
if [ -z "$BIN" ]; then
    echo "BlockVerse is not built. Run: cd crates/bv-server && cargo build --release"
    exit 1
fi
if command -v termux-wake-lock >/dev/null 2>&1; then
    termux-wake-lock
fi
mkdir -p "$DIR/bvmc"
cd "$DIR/bvmc" || exit 1
exec "$BIN" "$@"
