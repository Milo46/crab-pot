#!/bin/bash

curl \
    -H "Authorization: Bearer ${API_KEY}" \
    "$@"
