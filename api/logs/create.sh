#!/bin/bash

source "$(dirname "$0")/../config.sh"

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

PAYLOAD=$(cat <<EOF
{
    "schema_id": "5d8f6e7a-282a-4e35-b1ea-403aa99b5763",
    "log_data": {
        "level": "DEBUG",
        "message": "Hello, World at $TIMESTAMP",
        "request_id": "localtest"
    }
}
EOF
)

request POST "/logs" "$PAYLOAD"
