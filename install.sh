#!/usr/bin/env bash
set -euo pipefail

# web-action — one-line source installer
# Usage: curl -fsSL https://raw.githubusercontent.com/DdogezD/web-action/main/install.sh | bash

REPO="https://github.com/DdogezD/web-action.git"
BIN_DIR="${WEB_ACTION_BIN_DIR:-$HOME/.local/bin}"
CARGO_FLAGS="${WEB_ACTION_CARGO_FLAGS:---release}"

BOLD="\033[1m"
GREEN="\033[32m"
YELLOW="\033[33m"
RED="\033[31m"
RESET="\033[0m"

info()  { printf "${BOLD}${GREEN}→${RESET} %s\n" "$1"; }
warn()  { printf "${BOLD}${YELLOW}!${RESET} %s\n" "$1"; }
err()   { printf "${BOLD}${RED}✗${RESET} %s\n" "$1"; exit 1; }

# ── Prerequisites ──────────────────────────────────────────────

info "Checking prerequisites..."

command -v git    >/dev/null 2>&1 || err "git is required. Install it: sudo apt install git"
command -v cargo  >/dev/null 2>&1 || err "cargo (Rust) is required. Install it: https://rustup.rs"

# Check for chromium (preferred) or chrome
BROWSER=""
for candidate in chromium-browser chromium google-chrome; do
    if command -v "$candidate" >/dev/null 2>&1; then
        BROWSER="$candidate"
        break
    fi
done
if [ -z "$BROWSER" ]; then
    warn "No chromium/chrome found. Install it: sudo apt install chromium-browser"
    warn "web-action will fail to launch a browser until this is installed."
fi

# ── Clone & build ──────────────────────────────────────────────

BUILD_DIR="/tmp/web-action-build-$$"
trap 'rm -rf "$BUILD_DIR"' EXIT

info "Cloning $REPO..."
git clone --depth 1 "$REPO" "$BUILD_DIR"
cd "$BUILD_DIR"

info "Building web-action (this may take a few minutes)..."
cargo build $CARGO_FLAGS --manifest-path cli/Cargo.toml

# ── Install binary ─────────────────────────────────────────────

mkdir -p "$BIN_DIR"

# Find the built binary
if echo "$CARGO_FLAGS" | grep -q '\--release'; then
    BINARY="cli/target/release/web-action"
elif echo "$CARGO_FLAGS" | grep -q '\--profile'; then
    PROFILE=$(echo "$CARGO_FLAGS" | grep -oP '\-\-profile\s+\S+' | awk '{print $2}')
    BINARY="cli/target/$PROFILE/web-action"
else
    BINARY="cli/target/debug/web-action"
fi

if [ ! -f "$BINARY" ]; then
    err "Build failed: binary not found at $BINARY"
fi

# Replace the old binary. If it's running, `rm` may fail with ETXTBUSY;
# fall back to renaming the old file first (the running process keeps its inode).
if ! rm -f "$BIN_DIR/web-action" 2>/dev/null; then
    mv "$BIN_DIR/web-action" "$BIN_DIR/web-action.old" 2>/dev/null || true
fi
cp "$BINARY" "$BIN_DIR/web-action"
chmod +x "$BIN_DIR/web-action"
rm -f "$BIN_DIR/web-action.old"

# Copy Claude Code skill before BUILD_DIR is cleaned up by trap
SKILL_DIR="${CLAUDE_CODE_SKILL_DIR:-$HOME/.claude/skills/web-action}"
mkdir -p "$SKILL_DIR"
cp "$BUILD_DIR/skills/web-action/SKILL.md" "$SKILL_DIR/SKILL.md"

info "Installed web-action → $BIN_DIR/web-action"
info "Installed skill → $SKILL_DIR/SKILL.md"

# ── PATH check ─────────────────────────────────────────────────

if ! echo "$PATH" | tr ':' '\n' | grep -qxF "$BIN_DIR"; then
    warn "$BIN_DIR is not in your PATH."
    echo ""
    echo "  Add this to your ~/.bashrc or ~/.zshrc:"
    echo ""
    echo "    export PATH=\"$BIN_DIR:\$PATH\""
    echo ""
fi

# ── Verify ─────────────────────────────────────────────────────

if command -v web-action >/dev/null 2>&1; then
    VERSION=$(web-action --version 2>/dev/null || echo "unknown")
    info "web-action $VERSION is ready."
elif [ -x "$BIN_DIR/web-action" ]; then
    PATH="$BIN_DIR:$PATH" web-action --version 2>/dev/null && \
        info "web-action installed. Add $BIN_DIR to your PATH to use it." || \
        warn "web-action installed but failed to run. Check your system."
fi

echo ""
echo "  Quick test:"
echo "    web-action open https://example.com"
echo "    web-action snapshot -i"
