#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf -- "$TMP_DIR"' EXIT
mkdir -p "$TMP_DIR/package" "$TMP_DIR/home"
cp "$ROOT/target/release/forge" "$TMP_DIR/package/forge"
cp -R "$ROOT/exercises" "$TMP_DIR/package/exercises"
tar -czf "$TMP_DIR/forge.tar.gz" -C "$TMP_DIR/package" .

cd "$TMP_DIR"
HOME="$TMP_DIR/home" FORGE_ARCHIVE="$TMP_DIR/forge.tar.gz" FORGE_INSTALL_DIR="$TMP_DIR/data" FORGE_BIN_DIR="$TMP_DIR/bin" bash "$ROOT/install.sh"
HOME="$TMP_DIR/home" "$TMP_DIR/bin/forge" doctor
HOME="$TMP_DIR/home" "$TMP_DIR/bin/forge" list --language rust > "$TMP_DIR/list.txt"
rg -q '^ID[[:space:]]+LANG' "$TMP_DIR/list.txt"
