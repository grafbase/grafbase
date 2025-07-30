#!/usr/bin/env bash

set -euo pipefail

script_dir="$(dirname "$(readlink -f "$0")")"
cd "$script_dir"

cargo clean
for dir in crates/*; do
    pushd "$dir"
    rm -rf build cache target
    popd
done
