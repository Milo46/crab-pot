#!/bin/bash

api-curl() {
    curl \
        -H "Authorization: Bearer ${API_KEY}" \
        "$@"
}
