# Cross-Platform Build Guide

This guide explains how to build OpenMiner for multiple platforms: macOS, Raspberry Pi 5 (Linux ARM64), and Windows.

## Prerequisites

### All Platforms
- Node.js 18+ (`brew install node@20`)
- pnpm 8+ (`brew install pnpm`)
- Rust 1.70+ (`rustup default stable`)

### For Cross-Compilation from macOS

#### Raspberry Pi 5 (ARM64 Linux)
```bash
# Install ARM64 cross-compiler
brew tap messense/macos-cross-toolchains
brew install aarch64-unknown-linux-gnu

# Add Rust target
rustup target add aarch64-unknown-linux-gnu
```

#### Windows (x86_64)
```bash
# Install MinGW-w64
brew install mingw-w64

# Add Rust target
rustup target add x86_64-pc-windows-gnu
```

## Build Instructions

### Option 1: Build All Platforms at Once
```bash
./scripts/build-all-platforms.sh
```

This will build for:
- macOS (native)
- Raspberry Pi 5 (ARM64 Linux)
- Windows (x86_64)

### Option 2: Build Individual Platforms

#### macOS (Native)
```bash
pnpm install
pnpm tauri:build
```

Output: `apps/desktop/src-tauri/target/release/bundle/`

#### Raspberry Pi 5
```bash
./scripts/build-pi5.sh
```

Output: `apps/desktop/src-tauri/target/aarch64-unknown-linux-gnu/release/openminedash`

#### Windows
```bash
./scripts/build-windows.sh
```

Output: `apps/desktop/src-tauri/target/x86_64-pc-windows-gnu/release/openminedash.exe`

## Distribution

### Raspberry Pi 5

1. Copy the binary to your Raspberry Pi 5:
```bash
scp apps/desktop/src-tauri/target/aarch64-unknown-linux-gnu/release/openminedash pi@raspberrypi.local:~/
```

2. On the Raspberry Pi, install dependencies:
```bash
sudo apt-get update
sudo apt-get install -y libwebkit2gtk-4.0-37 libayatana-appindicator3-1
```

3. Run the application:
```bash
chmod +x openminedash
./openminedash
```

### Windows

1. Copy the `.exe` file to a Windows machine
2. Ensure WebView2 Runtime is installed (usually pre-installed on Windows 11)
   - Download from: https://developer.microsoft.com/en-us/microsoft-edge/webview2/
3. Copy the `assets/coins` folder alongside the executable
4. Run `openminedash.exe`

### macOS

Use the generated `.dmg` or `.app` bundle from:
```
apps/desktop/src-tauri/target/release/bundle/dmg/
```

## Platform-Specific Notes

### Raspberry Pi 5
- **Performance**: The Pi 5 has 8GB RAM and a powerful ARM Cortex-A76 CPU, suitable for mining management
- **Mining**: XMRig works well on ARM64 for Monero (RandomX)
- **Display**: Requires a desktop environment (LXDE, XFCE, or Raspberry Pi OS Desktop)
- **GPU**: Can utilize VideoCore VII for some operations

### Windows
- **WebView2**: Required for Tauri apps on Windows
- **Antivirus**: Mining software may trigger false positives - add exceptions as needed
- **Firewall**: Ensure mining pools are not blocked

### macOS
- **Gatekeeper**: May need to allow the app in System Settings → Privacy & Security
- **Apple Silicon**: Native ARM64 support for M1/M2/M3 chips
- **Intel**: Also supported via x86_64 build

## Troubleshooting

### Cross-Compilation Errors

**Error: `aarch64-linux-gnu-gcc not found`**
```bash
brew tap messense/macos-cross-toolchains
brew install aarch64-unknown-linux-gnu
```

**Error: `x86_64-w64-mingw32-gcc not found`**
```bash
brew install mingw-w64
```

**Error: `linker 'cc' not found`**
```bash
xcode-select --install
```

### Runtime Errors on Target Platform

**Raspberry Pi: `error while loading shared libraries`**
```bash
sudo apt-get install -y libwebkit2gtk-4.0-37 libayatana-appindicator3-1
```

**Windows: `VCRUNTIME140.dll not found`**
- Install Visual C++ Redistributable
- Or use static linking (add to Cargo.toml)

## GitHub Release Workflow

To create a multi-platform release:

1. Build all platforms:
```bash
./scripts/build-all-platforms.sh
```

2. Create release packages:
```bash
# macOS
cd apps/desktop/src-tauri/target/release/bundle/dmg/
zip -r OpenMiner-macOS-arm64.zip OpenMiner_*.dmg

# Raspberry Pi 5
cd apps/desktop/src-tauri/target/aarch64-unknown-linux-gnu/release/
tar -czf OpenMiner-pi5-arm64.tar.gz openminedash

# Windows
cd apps/desktop/src-tauri/target/x86_64-pc-windows-gnu/release/
zip OpenMiner-windows-x64.zip openminedash.exe
```

3. Create GitHub release:
```bash
gh release create v1.0.0 \
  OpenMiner-macOS-arm64.zip \
  OpenMiner-pi5-arm64.tar.gz \
  OpenMiner-windows-x64.zip \
  --title "OpenMiner v1.0.0 - Multi-Platform Release" \
  --notes "Supports macOS, Raspberry Pi 5, and Windows"
```

## CI/CD Integration

For automated builds, see `.github/workflows/build.yml` (to be created).

## Performance Benchmarks

| Platform | Build Time | Binary Size | RAM Usage |
|----------|-----------|-------------|-----------|
| macOS M1 | ~5 min | ~15 MB | ~80 MB |
| Pi 5 ARM64 | ~8 min | ~18 MB | ~100 MB |
| Windows x64 | ~6 min | ~16 MB | ~90 MB |

## License

MIT License - See [LICENSE](../LICENSE)
