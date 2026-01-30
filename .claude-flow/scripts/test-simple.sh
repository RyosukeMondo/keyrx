#!/usr/bin/env bash
set -x
echo "Script starting..."
PWD=$(pwd)
echo "Current directory: $PWD"

MODULES=("keyrx_core" "keyrx_daemon")
echo "Modules: ${MODULES[@]}"
echo "First module: ${MODULES[0]}"

STATE_DIR=".claude-flow/state"
mkdir -p "$STATE_DIR"
echo "Created state dir"

STATE_FILE="$STATE_DIR/test.txt"
echo "0" > "$STATE_FILE"
echo "Created state file"

CURRENT_INDEX=$(cat "$STATE_FILE")
echo "Current index: $CURRENT_INDEX"

CURRENT_MODULE="${MODULES[$CURRENT_INDEX]}"
echo "Current module: $CURRENT_MODULE"

if [ -d "$CURRENT_MODULE" ]; then
    echo "Module exists: $CURRENT_MODULE"
else
    echo "Module NOT found: $CURRENT_MODULE"
fi

echo "{\"success\": true, \"module\": \"$CURRENT_MODULE\"}"
