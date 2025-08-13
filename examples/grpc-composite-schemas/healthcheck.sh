#!/usr/bin/env bash

set -euo pipefail

# Configuration
URL="http://localhost:5000/health"
TIMEOUT=3
MAX_ATTEMPTS=60

attempt=1

while true; do
    echo "Attempt $attempt: Checking $URL..."

    # Use curl to check the endpoint
    response=$(curl -s -o /dev/null -w "%{http_code}" --max-time $TIMEOUT $URL || true)

    # Check if the status code indicates success (2xx)
    if [[ $response =~ ^2[0-9][0-9]$ ]]; then
        echo "Success! Received status code: $response"
        exit 0
    else
        # Check if we've reached the maximum number of attempts
        if [ "$attempt" -ge "$MAX_ATTEMPTS" ]; then
            echo "Maximum attempts reached. Giving up."
            exit 1
        fi
        sleep 1
        ((attempt++))
    fi
done