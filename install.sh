#!/usr/bin/env bash
# Install Forge from a GitHub release, or build from source when no binary exists.
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
  *) TARGET=source ;;
esac

TMP_DIR="$(mktemp -d)"
PACKAGE="$TMP_DIR/package"
mkdir -p "$PACKAGE"

if [ -n "${FORGE_ARCHIVE:-}" ]; then
  cp -- "$FORGE_ARCHIVE" "$TMP_DIR/forge.tar.gz"
  tar -xzf "$TMP_DIR/forge.tar.gz" -C "$PACKAGE"
elif [ "$TARGET" != source ]; then
  if [ "$VERSION" = latest ]; then
    URL="https://github.com/$REPO/releases/latest/download/forge-$TARGET.tar.gz"
  else
    URL="https://github.com/$REPO/releases/download/$VERSION/forge-$TARGET.tar.gz"
  fi
  say "Downloading Forge ($TARGET, $VERSION)..."
  if curl --proto '=https' --tlsv1.2 -fsSL --retry 3 --connect-timeout 15 "$URL" -o "$TMP_DIR/forge.tar.gz"; then
    tar -xzf "$TMP_DIR/forge.tar.gz" -C "$PACKAGE" || die "Release archive is damaged."
  else
    say "No release archive available; building from source."
  fi
fi

if [ ! -f "$PACKAGE/forge" ] || [ ! -d "$PACKAGE/exercises" ]; then
  require git
  require cargo
  if [ -f Cargo.toml ] && [ -d exercises ]; then
    SOURCE_DIR="$PWD"
  else
    SOURCE_DIR="$TMP_DIR/source"
    git clone --depth 1 "https://github.com/$REPO.git" "$SOURCE_DIR"
  fi
  say "Building Forge from source..."
  (cd "$SOURCE_DIR" && cargo build --release --locked)
  install -m 755 "$SOURCE_DIR/target/release/forge" "$PACKAGE/forge"
  cp -R "$SOURCE_DIR/exercises" "$PACKAGE/exercises"
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
