#!/usr/bin/env bash

set -euo pipefail

script_dir="$(dirname "$(readlink -f "$0")")"
cd "$script_dir"

for dir in crates/*; do
    pushd "$dir"
    ../../../../../../target/debug/grafbase extension build
    popd
done
