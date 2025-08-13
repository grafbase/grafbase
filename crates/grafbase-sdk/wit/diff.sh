#!/usr/bin/env bash

set -euo pipefail

if [ $# -ne 2 ]; then
    echo "Usage: $0 <old_sdk> <new_sdk>"
    echo "Compare SDK changes"
    exit 1
fi

old="since_${1//./_}"
new="since_${2//./_}"

find "$new/" -type f -exec bash -c '
      file="$1"
      old="$2"
      new="$3"
      relative="${file#$new/}"
      if [ -f "$old/$relative" ]; then
          if ! cmp -s "$old/$relative" "$file"; then
              difft "$old/$relative" "$file"
          fi
      fi
  ' _ {} "$old" "$new" \;
