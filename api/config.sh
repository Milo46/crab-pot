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

request() {
    local METHOD=$1
    local R_PATH=$2
    local BODY=$3
 
    if [ -n "$BODY" ]; then
        curl --silent --show-error --fail \
            --request "$METHOD" \
            --location "$BASE_URL$R_PATH" \
            --header "Content-Type: application/json" \
            --data "$BODY"
    else
        curl --silent --show-error --fail \
            --request "$METHOD" \
            --location "$BASE_URL$R_PATH" \
            --header "Content-Type: application/json"
    fi
}

