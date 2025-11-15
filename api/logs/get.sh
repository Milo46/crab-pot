#!/bin/bash

source "$(dirname "$0")/../config.sh"

SCHEMA_NAME=$1
SCHEMA_VERSION=$2

if [ -z "$SCHEMA_NAME" ]; then
    echo "Usage: $0 <schema_name> [schema_version]"
    exit 0
fi

if [ -z "$SCHEMA_VERSION" ]; then
    REQUEST_PATH="/logs/schema/$SCHEMA_NAME"
else
    REQUEST_PATH="/logs/schema/$SCHEMA_NAME/$SCHEMA_VERSION"
fi

request GET "$REQUEST_PATH"
