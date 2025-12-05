#!/bin/bash

: "${WORKFLOW:=dev}"

case $WORKFLOW in
    prod)
        HOST="localhost"
        PORT=8080
        ;;
    dev)
        HOST="localhost"
        PORT=8081
        ;;
    *)
        echo "Unknown WORKFLOW: $WORKFLOW"
        exit 1
        ;;
esac

BASE_URL="http://$HOST:$PORT"

build_query_params() {
    local params=()
    for param in "$@"; do
        params+=("$param")
    done
    IFS='&'
    echo "${params[*]}"
    unset IFS
}

request() {
    local METHOD=$1
    local R_PATH=$2
    local BODY=$3
    local QUERY_PARAMS=$4
    
    local URL="$BASE_URL$R_PATH"
    if [ -n "$QUERY_PARAMS" ]; then
        URL="$URL?$QUERY_PARAMS"
    fi
 
    if [ -n "$BODY" ]; then
        curl --silent --show-error --fail \
            --request "$METHOD" \
            --location "$URL" \
            --header "Content-Type: application/json" \
            --data "$BODY"
    else
        curl --silent --show-error --fail \
            --request "$METHOD" \
            --location "$URL" \
            --header "Content-Type: application/json"
    fi
}

