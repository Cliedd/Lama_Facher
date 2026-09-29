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

mkdir -p "$TMP_DIR/release" "$TMP_DIR/mock-bin"
cp "$TMP_DIR/forge.tar.gz" "$TMP_DIR/release/forge-linux-x86_64.tar.gz"
(
  cd "$TMP_DIR/release"
  sha256sum forge-linux-x86_64.tar.gz > SHA256SUMS
)
cp "$ROOT/scripts/mock-release-curl.sh" "$TMP_DIR/mock-bin/curl"
chmod +x "$TMP_DIR/mock-bin/curl"
PATH="$TMP_DIR/mock-bin:$PATH" HOME="$TMP_DIR/home" FORGE_TEST_RELEASE_DIR="$TMP_DIR/release" FORGE_INSTALL_DIR="$TMP_DIR/verified-data" FORGE_BIN_DIR="$TMP_DIR/verified-bin" bash "$ROOT/install.sh"
HOME="$TMP_DIR/home" "$TMP_DIR/verified-bin/forge" doctor

printf 'corrupted' >> "$TMP_DIR/release/forge-linux-x86_64.tar.gz"
if PATH="$TMP_DIR/mock-bin:$PATH" HOME="$TMP_DIR/home" FORGE_TEST_RELEASE_DIR="$TMP_DIR/release" FORGE_INSTALL_DIR="$TMP_DIR/rejected-data" FORGE_BIN_DIR="$TMP_DIR/rejected-bin" bash "$ROOT/install.sh" > "$TMP_DIR/rejected.log" 2>&1; then
  printf 'Corrupt release was installed.\n' >&2
  exit 1
fi
rg -q 'checksum mismatch' "$TMP_DIR/rejected.log"
[ ! -e "$TMP_DIR/rejected-bin/forge" ]
