#!/bin/bash
set -euo pipefail

MODULE=$1
COMMAND=$2
shift 2

SCRIPT="./api/$MODULE/$COMMAND.sh"

if [ ! -x "$SCRIPT" ]; then
    echo "Error: command '$MODULE $COMMAND' does not exist or is not executable."
    exit 1
fi

"$SCRIPT" "$@"
