#!/usr/bin/env bash

set -euo pipefail

for dir in crates/*; do
    pushd "$dir"
    ../../../../../../target/debug/grafbase extension build
    popd
done
