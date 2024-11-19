#!/usr/bin/env bash

set -euo pipefail

# Simple test script that run all the composition tests that failed individually asking whether the output should be updated.
# It asks a second time for the sdl roundtrip test that might need a second update.

RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

printf "Running all composition tests...\n"

for test in $(cargo nextest run -p graphql-composition --no-fail-fast --failure-output never 2>&1 | rg FAIL | rg run_test:: | sd '.*run_test::(.*)$' '$1'); do
    test="run_test::$test"
    printf "%b\n===***=== Running  $test ===***===%b\n\n" "$RED" "$NC"
    if ! cargo test -p graphql-composition "$test"; then
        read -p "$(printf "%b\nUpdate ([aA] for accept)?%b " "$BLUE" "$NC")" -n 1 -r
        echo
        if [[ $REPLY =~ ^[Aa]$ ]]; then
            if ! UPDATE_EXPECT=1 cargo test -p graphql-composition "$test"; then
                read -p "$(printf "%b\nUpdate again ([aA] for accept)?%b " "$BLUE" "$NC")" -n 1 -r
                echo
                if [[ $REPLY =~ ^[Aa]$ ]]; then
                    UPDATE_EXPECT=1 cargo test -p graphql-composition "$test"
                fi
            fi
        fi
    fi
done
