#!/bin/bash
# rubox convenience wrapper script

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BINARY="$SCRIPT_DIR/apps/rubox/target/release/rubox"

# Check if binary exists
if [ ! -f "$BINARY" ]; then
    echo "Error: rubox binary not found at $BINARY"
    echo "Please run: cd apps/rubox && cargo build --release"
    exit 1
fi

# Change to app directory for relative paths to work
cd "$SCRIPT_DIR/apps/rubox"

# Run rubox with all arguments passed through
exec "$BINARY" "$@"
