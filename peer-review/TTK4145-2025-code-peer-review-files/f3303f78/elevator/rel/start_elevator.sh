#!/bin/sh
set -e


DIR="$(cd "$(dirname "$0")" && pwd)"
SCRIPT_DIR="$(basename "$DIR")"
cd "$DIR/bin"

if [ -f "$SCRIPT_DIR" ]; then
    echo "Executing $SCRIPT_DIR..."
    ./"$SCRIPT_DIR" start
    exit 0
else
    echo "No application file named $SCRIPT_DIR found in the bin directory."
    exit 1
fi
