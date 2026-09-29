#!/usr/bin/env bash
# Install a verified Forge release without requiring a Rust toolchain.
set -euo pipefail

REPO="${FORGE_REPO:-Cliedd/Lama_Facher}"
INSTALL_DIR="${FORGE_INSTALL_DIR:-${HOME}/.local/share/forge}"
BIN_DIR="${FORGE_BIN_DIR:-${HOME}/.local/bin}"
VERSION="${FORGE_VERSION:-latest}"
TMP_DIR=""

say() { printf '%s\n' "$*"; }
die() { printf 'Forge: %s\n' "$*" >&2; exit 1; }
cleanup() { if [ -n "$TMP_DIR" ]; then rm -rf -- "$TMP_DIR"; fi; }
trap cleanup EXIT

require() { command -v "$1" >/dev/null 2>&1 || die "Missing $1. See https://github.com/$REPO#installation"; }
require curl
require tar
require mktemp
require install

case "$(uname -s)-$(uname -m)" in
  Linux-x86_64) TARGET=linux-x86_64 ;;
  Darwin-arm64) TARGET=macos-aarch64 ;;
  Darwin-x86_64) TARGET=macos-x86_64 ;;
  *) die "No prebuilt release for $(uname -s)-$(uname -m). See https://github.com/$REPO#development for a source build." ;;
esac

TMP_DIR="$(mktemp -d)"
PACKAGE="$TMP_DIR/package"
mkdir -p "$PACKAGE"

if [ -n "${FORGE_ARCHIVE:-}" ]; then
  cp -- "$FORGE_ARCHIVE" "$TMP_DIR/forge.tar.gz"
  tar -xzf "$TMP_DIR/forge.tar.gz" -C "$PACKAGE"
else
  ARCHIVE_NAME="forge-$TARGET.tar.gz"
  if [ "$VERSION" = latest ]; then
    BASE_URL="https://github.com/$REPO/releases/latest/download"
  else
    BASE_URL="https://github.com/$REPO/releases/download/$VERSION"
  fi
  say "Downloading Forge ($TARGET, $VERSION)..."
  if ! curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --connect-timeout 15 "$BASE_URL/$ARCHIVE_NAME" -o "$TMP_DIR/forge.tar.gz"; then
    die "Release archive unavailable. Check https://github.com/$REPO/releases or set FORGE_VERSION to a published tag."
  fi
  if ! curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --connect-timeout 15 "$BASE_URL/SHA256SUMS" -o "$TMP_DIR/SHA256SUMS"; then
    die "Release checksums unavailable; installation stopped."
  fi
  EXPECTED="$(awk -v filename="$ARCHIVE_NAME" '$2 == filename {print $1}' "$TMP_DIR/SHA256SUMS")"
  if [ "${#EXPECTED}" -ne 64 ]; then
    die "Missing or invalid checksum for $ARCHIVE_NAME."
  fi
  case "$EXPECTED" in
    *[!0-9a-fA-F]*) die "Invalid checksum for $ARCHIVE_NAME." ;;
  esac
  if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL="$(sha256sum "$TMP_DIR/forge.tar.gz" | awk '{print $1}')"
  elif command -v shasum >/dev/null 2>&1; then
    ACTUAL="$(shasum -a 256 "$TMP_DIR/forge.tar.gz" | awk '{print $1}')"
  else
    die "SHA-256 utility missing (sha256sum or shasum)."
  fi
  [ "$ACTUAL" = "$EXPECTED" ] || die "Archive checksum mismatch; installation stopped."
  tar -xzf "$TMP_DIR/forge.tar.gz" -C "$PACKAGE" || die "Release archive is damaged."
fi

[ -s "$PACKAGE/forge" ] || die "Package has no executable."
[ -d "$PACKAGE/exercises/java" ] && [ -d "$PACKAGE/exercises/rust" ] || die "Package has no Java/Rust exercises."

# Each install gets its own directory. The launcher switches only after all files exist.
mkdir -p "$INSTALL_DIR/releases" "$BIN_DIR"
RELEASE_DIR="$(mktemp -d "$INSTALL_DIR/releases/.install.XXXXXXXX")"
install -m 755 "$PACKAGE/forge" "$RELEASE_DIR/forge"
cp -R "$PACKAGE/exercises" "$RELEASE_DIR/exercises"
LAUNCHER="$TMP_DIR/forge-launcher"
printf '#!/usr/bin/env bash\nexport FORGE_HOME=%q\nexec %q "$@"\n' "$RELEASE_DIR" "$RELEASE_DIR/forge" > "$LAUNCHER"
install -m 755 "$LAUNCHER" "$BIN_DIR/.forge-new-$$"
mv -f "$BIN_DIR/.forge-new-$$" "$BIN_DIR/forge"

say "Forge installed. Run: forge doctor, then forge start"
case ":$PATH:" in
  *:"$BIN_DIR":*) ;;
  *) say "Add Forge to PATH for this shell: export PATH=\"$BIN_DIR:\$PATH\"" ;;
esac
say "Executable: $BIN_DIR/forge"
