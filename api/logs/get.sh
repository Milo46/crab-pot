#!/bin/bash

SCHEMA_NAME=$1
SCHEMA_VERSION=$2

if [ -z "$SCHEMA_NAME" ]; then
    echo "Usage: $0 <schema_name> [schema_version]"
    exit 0
fi

if [ -z "$SCHEMA_VERSION" ]; then
    URL="http://localhost:8081/logs/schema/$SCHEMA_NAME"
else
    URL="http://localhost:8081/logs/schema/$SCHEMA_NAME/$SCHEMA_VERSION"
fi

curl \
    --request GET \
    --location $URL \
    --header "Content-Type: application/json"
