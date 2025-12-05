#!/bin/bash

source "$(dirname "$0")/../config.sh"

SCHEMA_NAME=$1
SCHEMA_VERSION=$2

PARAMS=()
[ -n "$SCHEMA_NAME" ] && PARAMS+=("name=$SCHEMA_NAME")
[ -n "$SCHEMA_VERSION" ] && PARAMS=("version=$SCHEMA_NAME")

IFS='&'
QUERY_PARAMS="${PARAMS[*]}"
unset IFS

request GET "/schemas" "" "$QUERY_PARAMS"
