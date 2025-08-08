#!/usr/bin/env bash

set -euo pipefail

if [ $# -ne 2 ]; then
    echo "Usage: $0 <old_sdk> <new_sdk>"
    echo "Compare SDK changes"
    exit 1
fi

old="since_$1"
new="since_$2"

find "$new/" -type f -exec bash -c '
      file="$1"
      old="$2"
      new="$3"
      relative="${file#$new/}"
      [ -f "$old/$relative" ] && difft "$old/$relative" "$file"
  ' _ {} "$old" "$new" \;
