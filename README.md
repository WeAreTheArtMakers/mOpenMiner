# OpenMiner

A transparent, open-source mining dashboard and manager for macOS (Apple Silicon).

## ⚠️ Important Notice

This application is a **mining orchestrator**, not a miner itself. It manages legitimate, open-source mining software (like XMRig) with full user control and transparency.

- **No hidden mining** - Requires explicit user consent
- **Default OFF** - Mining never starts automatically
- **One-click stop** - Instant termination always available
- **No auto-start** - No launchd agents, login items, or background daemons
- **Clean uninstall** - Removing the app removes everything

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

- macOS 12+ (Apple Silicon recommended)
- For XMR: XMRig binary (auto-downloaded with verification, or manual install)
- For BTC/LTC: External ASIC miner hardware

## Installation

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

- macOS 12+ (Apple Silicon recommended)
- Node.js 18+ (`brew install node@20`)
- pnpm 8+ (`brew install pnpm`)
- Rust 1.70+ (`rustup default stable`)
- Xcode Command Line Tools (`xcode-select --install`)

For detailed setup instructions, see [docs/LOCAL_DEVELOPMENT.md](docs/LOCAL_DEVELOPMENT.md).

### XMRig Binary Setup

**Option 1: Automatic download (recommended)**
- App downloads from official GitHub releases
- Verifies SHA256 against pinned checksums
- Handles macOS quarantine automatically

**Option 2: Manual install**
- Download XMRig from https://github.com/xmrig/xmrig/releases
- Go to Settings → Binary Path → Select your xmrig binary
- Useful for enterprise environments or custom builds

### macOS Gatekeeper Note

If macOS blocks the XMRig binary:
1. Go to System Settings → Privacy & Security
2. Find the blocked app message
3. Click "Allow Anyway"

Or use the manual install option with a signed binary.

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
