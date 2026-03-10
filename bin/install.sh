#!/usr/bin/env bash
# Standalone installer for rinda-cli and rinda-mcp.
# Downloads both binaries from GitHub Releases to ~/.rinda/bin/.
#
# Usage (standalone, no clone required):
#   bash <(curl -fsSL https://raw.githubusercontent.com/FINGU-GRINDA/claude-rinda-plugin/main/bin/install.sh)
#
# Usage (inside plugin hooks, where plugin.json is available):
#   This script auto-detects whether it's running inside the plugin tree and
#   reads the pinned version from plugin.json when available; otherwise it
#   fetches the latest release tag from the GitHub API.
set -euo pipefail

REPO="FINGU-GRINDA/claude-rinda-plugin"
INSTALL_DIR="$HOME/.rinda/bin"
mkdir -p "$INSTALL_DIR"

# ── Version resolution ────────────────────────────────────────────────────────
# Try to read the pinned version from the plugin.json that lives alongside
# this installer inside a checked-out repo.  Fall back to the GitHub API
# "latest release" tag when running as a standalone curl | bash.

resolve_version() {
  local script_dir
  script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || echo "")"

  # Walk up looking for .claude-plugin/plugin.json (works inside checked-out repo).
  if [ -n "$script_dir" ]; then
    local dir="$script_dir"
    while [ "$dir" != "/" ]; do
      if [ -f "$dir/.claude-plugin/plugin.json" ]; then
        local v
        v=$(python3 -c "import json; print(json.load(open('$dir/.claude-plugin/plugin.json'))['version'])" 2>/dev/null \
          || node -e "console.log(JSON.parse(require('fs').readFileSync('$dir/.claude-plugin/plugin.json','utf8')).version)" 2>/dev/null \
          || true)
        if [ -n "$v" ]; then
          echo "$v"
          return
        fi
      fi
      dir="$(dirname "$dir")"
    done
  fi

  # Fallback: query the GitHub Releases API for the latest rinda-plugin tag.
  local api_url="https://api.github.com/repos/${REPO}/releases"
  local tag
  tag=$(curl -fsSL "$api_url" 2>/dev/null \
    | python3 -c "import json,sys; releases=[r for r in json.load(sys.stdin) if r['tag_name'].startswith('rinda-plugin-v')]; print(releases[0]['tag_name'].removeprefix('rinda-plugin-v')) if releases else sys.exit(1)" 2>/dev/null \
    || curl -fsSL "$api_url" 2>/dev/null \
    | node -e "const d=JSON.parse(require('fs').readFileSync('/dev/stdin','utf8')); const r=d.filter(x=>x.tag_name.startsWith('rinda-plugin-v')); if(!r.length) process.exit(1); console.log(r[0].tag_name.replace('rinda-plugin-v',''))" 2>/dev/null \
    || true)

  if [ -z "$tag" ]; then
    echo "error: could not resolve version from plugin.json or GitHub API" >&2
    exit 1
  fi
  echo "$tag"
}

VERSION="$(resolve_version)"
TAG="rinda-plugin-v${VERSION}"

# ── Platform detection ────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux)               OS_SLUG="linux"   ;;
  Darwin)              OS_SLUG="macos"   ;;
  MINGW*|MSYS*|CYGWIN*) OS_SLUG="windows" ;;
  *) echo "error: unsupported OS: $OS" >&2; exit 1 ;;
esac

case "$ARCH" in
  x86_64|amd64)  ARCH_SLUG="x64"   ;;
  aarch64|arm64) ARCH_SLUG="arm64" ;;
  *) echo "error: unsupported architecture: $ARCH" >&2; exit 1 ;;
esac

CLI_ARTIFACT="rinda-cli-${OS_SLUG}-${ARCH_SLUG}"
MCP_ARTIFACT="rinda-mcp-${OS_SLUG}-${ARCH_SLUG}"
CLI_BINARY="$INSTALL_DIR/rinda-cli"
MCP_BINARY="$INSTALL_DIR/rinda-mcp"

# Windows binaries use .exe suffix.
if [ "$OS_SLUG" = "windows" ]; then
  CLI_ARTIFACT="${CLI_ARTIFACT}.exe"
  MCP_ARTIFACT="${MCP_ARTIFACT}.exe"
  CLI_BINARY="${CLI_BINARY}.exe"
  MCP_BINARY="${MCP_BINARY}.exe"
fi

# ── Download helper ───────────────────────────────────────────────────────────
download() {
  local url="$1" dest="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fSL --retry 3 -o "$dest" "$url"
  elif command -v wget >/dev/null 2>&1; then
    wget -q -O "$dest" "$url"
  else
    echo "error: curl or wget is required" >&2
    exit 1
  fi
}

# ── Install rinda-cli ─────────────────────────────────────────────────────────
SKIP_CLI=false
if [ -x "$CLI_BINARY" ]; then
  CURRENT=$("$CLI_BINARY" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)
  if [ "$CURRENT" = "$VERSION" ]; then
    echo "rinda-cli v${VERSION} already installed, skipping."
    SKIP_CLI=true
  else
    rm -f "$CLI_BINARY"
  fi
fi

if [ "$SKIP_CLI" = false ]; then
  CLI_URL="https://github.com/${REPO}/releases/download/${TAG}/${CLI_ARTIFACT}"
  echo "Downloading rinda-cli v${VERSION} for ${OS}/${ARCH}..."
  download "$CLI_URL" "$CLI_BINARY"
  chmod +x "$CLI_BINARY"
  echo "Installed rinda-cli v${VERSION} to $CLI_BINARY"
fi

# ── Install rinda-mcp ─────────────────────────────────────────────────────────
SKIP_MCP=false
if [ -x "$MCP_BINARY" ]; then
  MCP_CURRENT=$("$MCP_BINARY" --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' || true)
  if [ "$MCP_CURRENT" = "$VERSION" ]; then
    echo "rinda-mcp v${VERSION} already installed, skipping."
    SKIP_MCP=true
  else
    rm -f "$MCP_BINARY"
  fi
fi

if [ "$SKIP_MCP" = false ]; then
  MCP_URL="https://github.com/${REPO}/releases/download/${TAG}/${MCP_ARTIFACT}"
  echo "Downloading rinda-mcp v${VERSION} for ${OS}/${ARCH}..."
  download "$MCP_URL" "$MCP_BINARY"
  chmod +x "$MCP_BINARY"
  echo "Installed rinda-mcp v${VERSION} to $MCP_BINARY"
fi

# ── PATH hint ─────────────────────────────────────────────────────────────────
echo ""
echo "Done! Both binaries are in $INSTALL_DIR"
echo ""
if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
  echo "Add the following to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
  echo "  export PATH=\"\$PATH:$INSTALL_DIR\""
  echo ""
fi
echo "Next step: run 'rinda-cli auth login' to authenticate with RINDA."
