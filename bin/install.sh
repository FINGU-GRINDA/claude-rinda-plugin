#!/usr/bin/env bash
# Download the rinda-cli binary from GitHub Releases.
# Detects OS/arch and fetches the version pinned in .release-please-manifest.json.
set -euo pipefail

REPO="FINGU-GRINDA/claude-rinda-plugin"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PLUGIN_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
MANIFEST="$PLUGIN_ROOT/.release-please-manifest.json"
BINARY="$SCRIPT_DIR/rinda-cli"

# Read pinned version from manifest.
if [ ! -f "$MANIFEST" ]; then
  echo "error: .release-please-manifest.json not found" >&2
  exit 1
fi

VERSION=$(python3 -c "import json; print(json.load(open('$MANIFEST'))['crates/cli'])" 2>/dev/null \
  || node -e "console.log(JSON.parse(require('fs').readFileSync('$MANIFEST','utf8'))['crates/cli'])" 2>/dev/null)

if [ -z "$VERSION" ]; then
  echo "error: could not read version from manifest" >&2
  exit 1
fi

TAG="rinda-cli-v${VERSION}"

# Skip download if correct version already exists.
if [ -x "$BINARY" ]; then
  CURRENT=$("$BINARY" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)
  if [ "$CURRENT" = "$VERSION" ]; then
    exit 0
  fi
fi

# Detect OS.
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)   ARTIFACT="rinda-cli-linux" ;;
  Darwin)  ARTIFACT="rinda-cli-macos" ;;
  MINGW*|MSYS*|CYGWIN*) ARTIFACT="rinda-cli-windows" ;;
  *) echo "error: unsupported OS: $OS" >&2; exit 1 ;;
esac

# Detect architecture.
case "$ARCH" in
  x86_64|amd64)  ARTIFACT="${ARTIFACT}-x64" ;;
  aarch64|arm64) ARTIFACT="${ARTIFACT}-arm64" ;;
  *) echo "error: unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

# Windows binary has .exe suffix.
case "$OS" in
  MINGW*|MSYS*|CYGWIN*) ARTIFACT="${ARTIFACT}.exe" ;;
esac

URL="https://github.com/${REPO}/releases/download/${TAG}/${ARTIFACT}"

echo "Downloading rinda-cli v${VERSION} for ${OS}/${ARCH}..."
if command -v curl >/dev/null 2>&1; then
  curl -fSL --retry 3 -o "$BINARY" "$URL"
elif command -v wget >/dev/null 2>&1; then
  wget -q -O "$BINARY" "$URL"
else
  echo "error: curl or wget required" >&2
  exit 1
fi

chmod +x "$BINARY"
echo "Installed rinda-cli v${VERSION} to $BINARY"
