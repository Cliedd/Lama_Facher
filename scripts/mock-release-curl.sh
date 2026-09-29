#!/usr/bin/env bash
set -euo pipefail

output=''
url=''
while [ "$#" -gt 0 ]; do
  case "$1" in
    -o) output="$2"; shift 2 ;;
    https://*) url="$1"; shift ;;
    *) shift ;;
  esac
done

[ -n "$output" ] && [ -n "$url" ]
cp -- "$FORGE_TEST_RELEASE_DIR/${url##*/}" "$output"
