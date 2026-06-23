#!/bin/bash
set -euo pipefail

# OpenMineDash Development Setup Script
# For macOS Apple Silicon (M1/M2/M3)
# 
# This script is for DEVELOPMENT ONLY.
# It does NOT:
# - Install any persistence mechanisms
# - Enable auto-start
# - Start mining without user consent

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${BOLD}╔════════════════════════════════════════╗${NC}"
echo -e "${BOLD}║   OpenMineDash Development Setup       ║${NC}"
echo -e "${BOLD}║   macOS Apple Silicon (arm64)          ║${NC}"
echo -e "${BOLD}╚════════════════════════════════════════╝${NC}"
echo ""

# -----------------------------------------------------------------------------
# Helper Functions
# -----------------------------------------------------------------------------

log_step() { echo -e "\n${BOLD}${CYAN}▶ $1${NC}"; }
log_ok() { echo -e "${GREEN}✓${NC} $1"; }
log_warn() { echo -e "${YELLOW}⚠${NC} $1"; }
log_err() { echo -e "${RED}✗${NC} $1"; }

check_command() {
    if command -v "$1" &> /dev/null; then
        local ver=$($1 --version 2>/dev/null | head -1 || $1 -V 2>/dev/null | head -1)
        log_ok "$1: $ver"
        return 0
    else
        log_err "$1 not found"
        return 1
    fi
}

# -----------------------------------------------------------------------------
# Step 1: Architecture & Prerequisites Check
# -----------------------------------------------------------------------------

log_step "Step 1: Checking System & Prerequisites"

# Architecture check
ARCH=$(uname -m)
if [[ "$ARCH" != "arm64" ]]; then
    log_warn "Not running on Apple Silicon ($ARCH)"
    log_warn "This guide is optimized for M1/M2/M3 Macs"
    echo "  Continue anyway? Some features may not work correctly."
    read -p "  Continue? [y/N] " -n 1 -r
    echo ""
    [[ ! $REPLY =~ ^[Yy]$ ]] && exit 1
else
    log_ok "Apple Silicon detected ($ARCH)"
fi

# macOS version
MACOS_VER=$(sw_vers -productVersion)
log_ok "macOS $MACOS_VER"

PREREQ_OK=true

# Xcode CLT
if xcode-select -p &> /dev/null; then
    log_ok "Xcode Command Line Tools installed"
else
    log_err "Xcode Command Line Tools not found"
    echo "    Run: xcode-select --install"
    PREREQ_OK=false
fi

# Node.js
if ! check_command node; then
    echo "    Install: brew install node@20"
    PREREQ_OK=false
fi

# pnpm
if ! check_command pnpm; then
    echo "    Install: brew install pnpm"
    PREREQ_OK=false
fi

# Rust
if ! check_command rustc; then
    echo "    Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    PREREQ_OK=false
fi

# Cargo
check_command cargo || PREREQ_OK=false

if [[ "$PREREQ_OK" == false ]]; then
    log_err "Prerequisites not met. Install missing tools and re-run."
    echo ""
    echo "Quick install (if Homebrew is installed):"
    echo "  brew install node@20 pnpm"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

# -----------------------------------------------------------------------------
# Step 2: Repository Check & Dependencies
# -----------------------------------------------------------------------------

log_step "Step 2: Installing Dependencies"

# Repo root check
if [[ ! -f "package.json" ]] || [[ ! -f "Cargo.toml" ]]; then
    log_err "Not in repository root (missing package.json or Cargo.toml)"
    echo "    cd /path/to/openminedash && ./scripts/dev-setup.sh"
    exit 1
fi
log_ok "Repository root detected"

# Clean cache if requested
if [[ "${CLEAN:-}" == "1" ]]; then
    log_warn "Cleaning caches..."
    pnpm store prune 2>/dev/null || true
    cargo clean 2>/dev/null || true
fi

# pnpm install
echo "Installing Node dependencies..."
if [[ -f "pnpm-lock.yaml" ]]; then
    pnpm install --frozen-lockfile
else
    pnpm install
fi
log_ok "Node dependencies installed"

# Cargo fetch
echo "Fetching Rust dependencies..."
cargo fetch
log_ok "Rust dependencies fetched"

# -----------------------------------------------------------------------------
# Step 3: Run Tests
# -----------------------------------------------------------------------------

log_step "Step 3: Running Tests"

echo "Running Rust tests..."
if cargo test --workspace; then
    log_ok "Rust tests passed"
else
    log_err "Rust tests failed"
    exit 1
fi

echo "Running TypeScript check..."
if pnpm typecheck; then
    log_ok "TypeScript check passed"
else
    log_err "TypeScript errors"
    exit 1
fi

echo "Running lint..."
if pnpm lint 2>/dev/null; then
    log_ok "Lint passed"
else
    log_warn "Lint warnings (non-blocking)"
fi

# -----------------------------------------------------------------------------
# Step 4: Miner Binaries Info
# -----------------------------------------------------------------------------

log_step "Step 4: Miner Binaries"

BIN_DIR="$HOME/Library/Application Support/openminedash/bin"
XMRIG_PATH="$BIN_DIR/xmrig"
CPUMINER_PATH="$BIN_DIR/cpuminer-opt"

echo "Binary directory: $BIN_DIR"
echo ""

check_binary() {
    local name=$1
    local path=$2
    
    if [[ -f "$path" ]]; then
        log_ok "$name found"
        
        # Architecture check
        local file_info=$(file "$path")
        if [[ "$file_info" == *"arm64"* ]]; then
            log_ok "  ARM64 binary (correct)"
        else
            log_warn "  Not ARM64: $file_info"
        fi
        
        # Quarantine check
        if xattr -l "$path" 2>/dev/null | grep -q "com.apple.quarantine"; then
            log_warn "  Quarantined by macOS"
            echo "      Fix: xattr -dr com.apple.quarantine \"$path\""
        else
            log_ok "  No quarantine"
        fi
        
        # Executable check
        if [[ -x "$path" ]]; then
            log_ok "  Executable"
        else
            log_warn "  Not executable"
            echo "      Fix: chmod +x \"$path\""
        fi
        return 0
    else
        log_warn "$name not found at $path"
        return 1
    fi
}

echo "--- XMRig (for Monero/RandomX) ---"
if ! check_binary "XMRig" "$XMRIG_PATH"; then
    echo ""
    echo "  Manual install:"
    echo "    mkdir -p \"$BIN_DIR\""
    echo "    # Download from: https://github.com/xmrig/xmrig/releases"
    echo "    # Choose: xmrig-*-macos-arm64.tar.gz"
    echo "    tar -xzf xmrig-*-macos-arm64.tar.gz"
    echo "    mv xmrig-*/xmrig \"$XMRIG_PATH\""
    echo "    chmod +x \"$XMRIG_PATH\""
    echo "    xattr -dr com.apple.quarantine \"$XMRIG_PATH\""
fi

echo ""
echo "--- cpuminer-opt (for BTC/Scrypt Try-Anyway) ---"
if ! check_binary "cpuminer-opt" "$CPUMINER_PATH"; then
    echo ""
    echo "  Must compile from source (no official macOS binaries):"
    echo "    brew install automake autoconf openssl@3"
    echo "    git clone https://github.com/JayDDee/cpuminer-opt.git /tmp/cpuminer-opt"
    echo "    cd /tmp/cpuminer-opt"
    echo "    ./autogen.sh"
    echo "    CFLAGS=\"-O3 -march=native\" ./configure --with-crypto=/opt/homebrew/opt/openssl@3"
    echo "    make -j\$(sysctl -n hw.ncpu)"
    echo "    mkdir -p \"$BIN_DIR\""
    echo "    cp cpuminer \"$CPUMINER_PATH\""
    echo "    chmod +x \"$CPUMINER_PATH\""
fi

echo ""
echo "Tip: You can also use Settings → Binary Path to select custom locations."

# -----------------------------------------------------------------------------
# Step 5: Ready
# -----------------------------------------------------------------------------

log_step "Step 5: Setup Complete"

echo ""
echo -e "${GREEN}${BOLD}✓ Development environment ready!${NC}"
echo ""
echo "To start:"
echo "  ${CYAN}pnpm tauri dev${NC}"
echo ""
echo "┌─────────────────────────────────────────────────────────────┐"
echo "│  First Run Checklist:                                       │"
echo "├─────────────────────────────────────────────────────────────┤"
echo "│  □ Consent dialog appears (mining disabled until accepted)  │"
echo "│  □ Start button disabled without consent                    │"
echo "│  □ Tray icon: left-click toggle, right-click menu           │"
echo "│  □ STOP button always at top of tray menu                   │"
echo "│  □ Settings → Notifications → Test works                    │"
echo "│  □ Crash recovery: kill -9 app, reopen → Resume? default No │"
echo "└─────────────────────────────────────────────────────────────┘"
echo ""
echo "Quick Smoke Test (after consent):"
echo "  1. Select XMR → pick a pool → enter wallet → Start"
echo "  2. Select BTC → enable 'Try Mining Anyway' → Start"
echo "  3. Verify hashrate/logs appear, STOP works"
echo ""
echo "Docs: docs/LOCAL_DEVELOPMENT.md"
echo ""

# -----------------------------------------------------------------------------
# Optional: Start Dev Server
# -----------------------------------------------------------------------------

if [[ "${AUTO_START:-}" == "1" ]]; then
    echo "Starting dev server (AUTO_START=1)..."
    exec pnpm tauri dev
fi

read -p "Start development server now? [y/N] " -n 1 -r
echo ""
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo ""
    echo "Starting Tauri dev server... (Ctrl+C to stop)"
    pnpm tauri dev
fi
