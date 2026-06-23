# OpenMiner

A transparent, open-source mining dashboard and manager for macOS, Raspberry Pi 5, and Windows.

## 🖥️ Platform Support

| Platform | Architecture | Status |
|----------|-------------|--------|
| macOS | Apple Silicon (ARM64) | ✅ Fully Supported |
| macOS | Intel (x86_64) | ✅ Fully Supported |
| Raspberry Pi 5 | ARM64 (Linux) | ✅ Fully Supported |
| Windows | x86_64 | ✅ Fully Supported |
| Linux | x86_64 | ⚠️ Experimental |

## ⚠️ Important Notice

This application is a **mining orchestrator**, not a miner itself. It manages legitimate, open-source mining software (like XMRig) with full user control and transparency.

- **No hidden mining** - Requires explicit user consent
- **Default OFF** - Mining never starts automatically
- **One-click stop** - Instant termination always available
- **No auto-start** - No launchd agents, login items, or background daemons
- **Clean uninstall** - Removing the app removes everything

## 🚀 Quick Links

- **[📥 Download Latest Release](https://github.com/WeAreTheArtMakers/mOpenMiner/releases/latest)** - Get pre-built binaries
- **[📖 Raspberry Pi 5 Guide](docs/RASPBERRY_PI5.md)** - Complete Pi5 setup
- **[📖 Windows Guide](docs/WINDOWS.md)** - Complete Windows setup
- **[🔧 Build Instructions](BUILD_INSTRUCTIONS.md)** - Build from source
- **[🌐 Cross-Platform Guide](docs/CROSS_PLATFORM_BUILD.md)** - Multi-platform builds
- **[📋 Changelog](CHANGELOG.md)** - Version history

## Data Storage Clarification

This app stores **non-sensitive configuration only**:
- ✅ Wallet addresses (public, not private keys)
- ✅ Pool URLs
- ✅ Worker names
- ✅ Thread/performance settings
- ✅ User preferences (theme, profiles)

**Never stored:**
- ❌ Private keys or seed phrases
- ❌ Passwords
- ❌ Personal information

Config location: `~/Library/Application Support/openminedash/config.json`

## Features

- 🎛️ Single-panel dashboard for mining management
- 💰 Multi-coin support via plugin system (JSON definitions)
- 📊 Real-time stats via XMRig HTTP API
- 📝 Live log viewer with filtering
- 🔒 Pinned checksum verification for miner binaries
- 🌐 Pool health checking (TCP + TLS handshake)
- ⚡ Performance presets: Eco / Balanced / Max
- 🔧 Manual binary path option for enterprise environments

##  ![openMiner](https://raw.githubusercontent.com/WeAreTheArtMakers/mOpenMiner/main/openMiner.png)


## Supported Coins

| Coin | Algorithm | Mining Method |
|------|-----------|---------------|
| Monero (XMR) | RandomX | CPU (XMRig) ✅ |
| Bitcoin (BTC) | SHA-256 | External ASIC / Try Anyway ⚠️ |
| Litecoin (LTC) | Scrypt | External ASIC / Try Anyway ⚠️ |
| Dogecoin (DOGE) | Scrypt | External ASIC / Try Anyway ⚠️ |

> **Note:** BTC/LTC/DOGE CPU mining is not practical. SHA-256 and Scrypt are dominated by ASIC hardware.
> 
> **"Try Mining Anyway" Mode:** For educational/experimental purposes, you can attempt CPU mining on these coins using cpuminer-opt. Expect extremely low hashrate with no profitability. This feature is for learning, not earning.

## Requirements

- **macOS**: macOS 12+ (Apple Silicon or Intel)
- **Raspberry Pi 5**: Raspberry Pi OS (64-bit) with desktop environment
- **Windows**: Windows 10/11 with WebView2 Runtime
- For XMR: XMRig binary (auto-downloaded with verification, or manual install)
- For BTC/LTC: External ASIC miner hardware

## Installation

### Download Pre-built Binaries

**[📥 Download Latest Release (v1.1.0)](https://github.com/WeAreTheArtMakers/mOpenMiner/releases/latest)**

Choose your platform:
- **macOS (Apple Silicon)**: `OpenMiner_*_aarch64.dmg` or `.app`
- **macOS (Intel)**: `OpenMiner_*_x64.dmg` or `.app`
- **Raspberry Pi 5 (ARM64 Linux)**: `openMiner-pi5-arm64.tar.gz`
- **Windows (x64)**: `openMiner-windows-x64.zip`

> **Note**: Builds are automatically generated via GitHub Actions for all platforms.

### Quick Start (Development)

```bash
# Clone repository
git clone https://github.com/WeAreTheArtMakers/mOpenMiner.git
cd openminedash

# Run setup script (checks prereqs, installs deps, runs tests)
./scripts/dev-setup.sh

# Or manually:
pnpm install
pnpm tauri dev
```

### Prerequisites

**All Platforms:**
- Node.js 18+ (`brew install node@20` on macOS)
- pnpm 8+ (`brew install pnpm` on macOS)
- Rust 1.70+ (`rustup default stable`)

**macOS:**
- Xcode Command Line Tools (`xcode-select --install`)

**Raspberry Pi 5:**
- Raspberry Pi OS (64-bit) with desktop environment
- 8GB RAM recommended

**Windows:**
- Visual Studio Build Tools or MinGW-w64
- WebView2 Runtime (usually pre-installed on Windows 11)

For detailed setup instructions, see [docs/LOCAL_DEVELOPMENT.md](docs/LOCAL_DEVELOPMENT.md).

### Building from Source

**For your current platform:**
```bash
pnpm install
pnpm tauri:build
```

**For all platforms (cross-compilation):**
```bash
./scripts/build-all-platforms.sh
```

**For specific platforms:**
```bash
# Raspberry Pi 5
./scripts/build-pi5.sh

# Windows
./scripts/build-windows.sh
```

See [docs/CROSS_PLATFORM_BUILD.md](docs/CROSS_PLATFORM_BUILD.md) for detailed cross-compilation instructions.

## Platform-Specific Setup

### 🥧 Raspberry Pi 5

**Quick Install:**
```bash
# Download and extract
wget https://github.com/WeAreTheArtMakers/mOpenMiner/releases/latest/download/openMiner-pi5-arm64.tar.gz
tar -xzf openMiner-pi5-arm64.tar.gz

# Install dependencies
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.0-37 libayatana-appindicator3-1

# Run
chmod +x openminedash
./openminedash
```

**System Requirements:**
- Raspberry Pi 5 (4GB or 8GB RAM)
- Raspberry Pi OS (64-bit) with desktop
- Active cooling recommended

**Performance:**
- Monero (XMR): ~1-2 KH/s
- Power: ~15W (4x more efficient than desktop)
- Ideal for 24/7 low-power mining

📖 **[Complete Pi5 Guide →](docs/RASPBERRY_PI5.md)**

---

### 🪟 Windows

**Quick Install:**
1. Download `openMiner-windows-x64.zip` from [releases](https://github.com/WeAreTheArtMakers/mOpenMiner/releases/latest)
2. Extract to a folder (e.g., `C:\OpenMiner`)
3. Install [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 11)
4. Run `openminedash.exe`

**Windows Defender:**
Mining software may trigger false positives. Add exclusions:
- Settings → Windows Security → Virus & threat protection → Exclusions
- Add the OpenMiner folder

**Firewall:**
Allow the app through Windows Firewall when prompted.

📖 **[Complete Windows Guide →](docs/WINDOWS.md)**

---

### 🍎 macOS

**Quick Install:**
1. Download the `.dmg` file from [releases](https://github.com/WeAreTheArtMakers/mOpenMiner/releases/latest)
2. Open the DMG and drag OpenMiner to Applications
3. Launch from Applications folder

**Gatekeeper:**
If macOS blocks the app:
- System Settings → Privacy & Security → Allow Anyway

---

### XMRig Binary Setup

**Option 1: Automatic download (recommended)**
- App downloads from official GitHub releases
- Verifies SHA256 against pinned checksums
- Handles macOS quarantine automatically

**Option 2: Manual install**
- Download XMRig from https://github.com/xmrig/xmrig/releases
- Go to Settings → Binary Path → Select your xmrig binary
- Useful for enterprise environments or custom builds

## What's New in v1.1.0 🎉

### Multi-Platform Support
- ✅ **Raspberry Pi 5** - Native ARM64 Linux support for low-power 24/7 mining
- ✅ **Windows** - Full Windows 10/11 support with native binary
- ✅ **macOS** - Continued excellent support for Apple Silicon and Intel

### Automated Builds
- 🤖 GitHub Actions CI/CD for all platforms
- 📦 Pre-built binaries for every release
- 🔒 Automated security checks and checksums

### Documentation
- 📖 Complete platform-specific guides
- 🔧 Cross-compilation instructions
- 💡 Performance tuning tips for each platform

[View Full Changelog →](CHANGELOG.md)

## Project Structure

```
openminedash/
├── apps/desktop/          # Tauri + React UI
├── crates/
│   ├── core/              # Config, process manager, telemetry
│   ├── miner_adapters/    # XMRig and other miner adapters
│   └── pools/             # Pool config, health check
├── assets/coins/          # Coin definition JSONs
└── docs/                  # Documentation
```



##  ![openMiner](https://raw.githubusercontent.com/WeAreTheArtMakers/mOpenMiner/main/OpenMiner-Earning.png) 


## License

MIT License - See [LICENSE](LICENSE)

## Created By

**WATAM (We Are The Art Makers)**  
[wearetheartmakers.com](https://wearetheartmakers.com)

## Security

See [SECURITY.md](SECURITY.md) for our security policy and anti-cryptojacking stance.

## Documentation

- [ARCHITECTURE.md](ARCHITECTURE.md) - System design
- [SECURITY.md](SECURITY.md) - Security policy
- [CONTRIBUTING.md](CONTRIBUTING.md) - How to contribute
- [docs/CROSS_PLATFORM_BUILD.md](docs/CROSS_PLATFORM_BUILD.md) - Multi-platform build guide
- [docs/MINERS.md](docs/MINERS.md) - XMRig vs cpuminer-opt guide
- [docs/LICENSING.md](docs/LICENSING.md) - Third-party license compliance
- [docs/TUNING.md](docs/TUNING.md) - Performance tuning guide
- [docs/SANDBOX.md](docs/SANDBOX.md) - macOS sandbox strategy

## Supply Chain Security

Releases include:
- SBOM (Software Bill of Materials) in CycloneDX format
- SLSA provenance attestation
- SHA256 checksums for all artifacts

Miner binaries use pinned checksums verified before execution.
