#!/usr/bin/env bash
# Download the rinda-cli binary from GitHub Releases.
# Detects OS/arch and fetches the version pinned in plugin.json.
set -euo pipefail

REPO="FINGU-GRINDA/claude-rinda-plugin"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Find plugin.json by walking up from script location.
find_plugin_json() {
  local dir="$SCRIPT_DIR"
  while [ "$dir" != "/" ]; do
    [ -f "$dir/.claude-plugin/plugin.json" ] && echo "$dir/.claude-plugin/plugin.json" && return
    dir="$(dirname "$dir")"
  done
  return 1
}

PLUGIN_JSON=$(find_plugin_json) || { echo "error: .claude-plugin/plugin.json not found" >&2; exit 1; }
INSTALL_DIR="$HOME/.rinda/bin"
mkdir -p "$INSTALL_DIR"
BINARY="$INSTALL_DIR/rinda-cli"

# Read pinned version from plugin.json.

VERSION=$(python3 -c "import json; print(json.load(open('$PLUGIN_JSON'))['version'])" 2>/dev/null \
  || node -e "console.log(JSON.parse(require('fs').readFileSync('$PLUGIN_JSON','utf8')).version)" 2>/dev/null)

if [ -z "$VERSION" ]; then
  echo "error: could not read version from manifest" >&2
  exit 1
fi

TAG="rinda-plugin-v${VERSION}"

# Skip download if correct version already exists.
if [ -x "$BINARY" ]; then
  CURRENT=$("$BINARY" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)
  if [ "$CURRENT" = "$VERSION" ]; then
    exit 0
  fi
  # Remove old version before downloading new one.
  rm -f "$BINARY"
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
  MINGW*|MSYS*|CYGWIN*)
    ARTIFACT="${ARTIFACT}.exe"
    BINARY="${BINARY}.exe"
    ;;
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

# ── Download rinda-mcp binary ─────────────────────────────────────────────────

MCP_BINARY="$INSTALL_DIR/rinda-mcp"

# Skip download if correct version already exists.
if [ -x "$MCP_BINARY" ]; then
  MCP_CURRENT=$("$MCP_BINARY" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)
  if [ "$MCP_CURRENT" = "$VERSION" ]; then
    exit 0
  fi
  rm -f "$MCP_BINARY"
fi

# Reuse OS/arch artifact base detected above, swap rinda-cli -> rinda-mcp.
MCP_ARTIFACT="${ARTIFACT/rinda-cli/rinda-mcp}"

MCP_URL="https://github.com/${REPO}/releases/download/${TAG}/${MCP_ARTIFACT}"

echo "Downloading rinda-mcp v${VERSION} for ${OS}/${ARCH}..."
if command -v curl >/dev/null 2>&1; then
  curl -fSL --retry 3 -o "$MCP_BINARY" "$MCP_URL"
elif command -v wget >/dev/null 2>&1; then
  wget -q -O "$MCP_BINARY" "$MCP_URL"
else
  echo "error: curl or wget required" >&2
  exit 1
fi

chmod +x "$MCP_BINARY"
echo "Installed rinda-mcp v${VERSION} to $MCP_BINARY"
