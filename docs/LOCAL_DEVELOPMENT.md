# Local Development Guide

Complete guide for running OpenMineDash on macOS Apple Silicon.

## Quick Start

```bash
# One command setup (from repo root)
./scripts/dev-setup.sh
```

## Prerequisites

### 1. Xcode Command Line Tools

```bash
# Check if installed
xcode-select -p

# Install if missing
xcode-select --install
```

### 2. Homebrew Dependencies

```bash
# Install Homebrew (if not installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Node.js and pnpm
brew install node@20 pnpm

# Verify versions
node -v    # Should be 18+
pnpm -v    # Should be 8+
```

### 3. Rust Toolchain

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart terminal or source profile
source ~/.zshrc

# Verify
rustc -V   # Should be 1.70+
cargo -V
```

## Repository Setup

```bash
# Clone (if not already)
git clone https://github.com/user/openminedash.git
cd openminedash

# Install dependencies
pnpm install

# Run tests
cargo test --workspace
pnpm typecheck
pnpm lint
```

## Running Development Server

```bash
pnpm tauri dev
```

First launch takes longer (Rust compilation). Subsequent launches are faster.

## Miner Binary Setup

Binaries are stored in:
```
~/Library/Application Support/openminedash/bin/
```

### XMRig (for Monero/RandomX)

```bash
# Create directory
mkdir -p ~/Library/Application\ Support/openminedash/bin

# Download from GitHub releases
# https://github.com/xmrig/xmrig/releases
# Choose: xmrig-6.x.x-macos-arm64.tar.gz

# Extract and install
tar -xzf xmrig-*-macos-arm64.tar.gz
mv xmrig-*/xmrig ~/Library/Application\ Support/openminedash/bin/

# Make executable
chmod +x ~/Library/Application\ Support/openminedash/bin/xmrig

# Remove quarantine (macOS security)
xattr -dr com.apple.quarantine ~/Library/Application\ Support/openminedash/bin/xmrig

# Verify architecture
file ~/Library/Application\ Support/openminedash/bin/xmrig
# Should show: Mach-O 64-bit executable arm64
```

### cpuminer-opt (for BTC/Scrypt Try-Anyway)

No official macOS binaries. Must compile from source:

```bash
# Install build dependencies
brew install automake autoconf openssl@3

# Clone and build
git clone https://github.com/JayDDee/cpuminer-opt.git /tmp/cpuminer-opt
cd /tmp/cpuminer-opt

./autogen.sh
CFLAGS="-O3 -march=native" ./configure --with-crypto=/opt/homebrew/opt/openssl@3
make -j$(sysctl -n hw.ncpu)

# Install
cp cpuminer ~/Library/Application\ Support/openminedash/bin/cpuminer-opt
chmod +x ~/Library/Application\ Support/openminedash/bin/cpuminer-opt

# Verify
file ~/Library/Application\ Support/openminedash/bin/cpuminer-opt
```

### Manual Binary Path

You can also use Settings → Binary Path to select binaries from any location.

## First Run Checklist

After launching with `pnpm tauri dev`:

### Consent Gate
- [ ] Consent dialog appears on first launch
- [ ] Mining is disabled until consent is granted
- [ ] Start button is disabled without consent

### Tray Icon
- [ ] Tray icon appears in menu bar
- [ ] Left-click: toggles window visibility
- [ ] Right-click: shows menu
- [ ] STOP is always at top of menu

### Notifications
- [ ] Settings → Notifications → Test Notification works
- [ ] Notifications are opt-in (disabled by default)

### Crash Recovery
```bash
# Test crash recovery
# 1. Start mining
# 2. Force kill the app
kill -9 $(pgrep -f openminedash)
# 3. Reopen app
# 4. Should see "Resume mining?" dialog
# 5. Default should be "No" (don't auto-resume)
```

## Quick Smoke Test

### XMR Mining (XMRig)
1. Accept consent
2. Select coin: Monero (XMR)
3. Select pool (or enter custom stratum URL)
4. Enter your XMR wallet address
5. Click Start
6. Verify: hashrate appears, logs stream, shares accepted

### BTC Try-Anyway (cpuminer-opt)
1. Select coin: Bitcoin (BTC)
2. Warning appears about ASIC dominance
3. Click "Try Mining Anyway"
4. Enter pool URL and wallet
5. Click Start
6. Verify: cpuminer-opt starts, logs appear
7. Note: Hashrate will be extremely low (expected)

### STOP Test
1. While mining, click STOP button
2. Verify: process terminates within 3 seconds
3. Verify: state returns to "Stopped"

## Troubleshooting

### Cache Issues

```bash
# Clean all caches and rebuild
pnpm store prune
cargo clean
rm -rf node_modules
pnpm install
cargo build
```

### Quarantine Errors

If macOS blocks the binary:

```bash
# Check quarantine status
xattr -l ~/Library/Application\ Support/openminedash/bin/xmrig

# Remove quarantine
xattr -dr com.apple.quarantine ~/Library/Application\ Support/openminedash/bin/xmrig

# Or allow in System Settings → Privacy & Security
```

### Wrong Architecture

```bash
# Check binary architecture
file ~/Library/Application\ Support/openminedash/bin/xmrig

# Should show: Mach-O 64-bit executable arm64
# If it shows x86_64, download the ARM64 version
```

### Rust Compilation Errors

```bash
# If CC environment variable causes issues
unset CC
cargo build

# Or explicitly use clang
export CC=clang
cargo build
```

### Port Conflicts

XMRig uses port 45580 for its HTTP API. If blocked:

```bash
# Check what's using the port
lsof -i :45580

# Kill if needed
kill -9 <PID>
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CLEAN=1` | Clean caches before setup | - |
| `AUTO_START=1` | Auto-start dev server | - |

Example:
```bash
CLEAN=1 ./scripts/dev-setup.sh
AUTO_START=1 ./scripts/dev-setup.sh
```

## Security Notes

- **No auto-start**: Mining never starts without explicit user action
- **No persistence**: App doesn't install launchd agents or login items
- **Consent required**: Mining disabled until user accepts consent dialog
- **One-click STOP**: Always available, terminates within 3 seconds
- **Clean uninstall**: Removing app removes everything

## See Also

- [MINERS.md](MINERS.md) - Miner software details
- [LICENSING.md](LICENSING.md) - Third-party license compliance
- [../SECURITY.md](../SECURITY.md) - Security policy
