# OpenMiner - Build Instructions

Quick reference for building OpenMiner on all supported platforms.

## 🚀 Quick Start

### Build for Current Platform (macOS)
```bash
pnpm install
pnpm tauri:build
```

### Build for All Platforms
```bash
./scripts/build-all-platforms.sh
```

## 📦 Platform-Specific Builds

### 🍎 macOS (Native)

**Requirements:**
- macOS 12+
- Xcode Command Line Tools
- Node.js 18+, pnpm, Rust

**Build:**
```bash
pnpm install
pnpm tauri:build
```

**Output:**
- DMG: `apps/desktop/src-tauri/target/release/bundle/dmg/`
- App: `apps/desktop/src-tauri/target/release/bundle/macos/`

---

### 🥧 Raspberry Pi 5 (ARM64 Linux)

**Requirements (on macOS):**
```bash
# Install ARM64 cross-compiler
brew tap messense/macos-cross-toolchains
brew install aarch64-unknown-linux-gnu

# Add Rust target
rustup target add aarch64-unknown-linux-gnu
```

**Build:**
```bash
./scripts/build-pi5.sh
```

**Output:**
- Binary: `apps/desktop/src-tauri/target/aarch64-unknown-linux-gnu/release/openminedash`

**Deploy to Pi:**
```bash
# Copy to Pi
scp apps/desktop/src-tauri/target/aarch64-unknown-linux-gnu/release/openminedash pi@raspberrypi.local:~/

# On Pi, install dependencies
ssh pi@raspberrypi.local
sudo apt-get install -y libwebkit2gtk-4.0-37 libayatana-appindicator3-1
chmod +x openminedash
./openminedash
```

---

### 🪟 Windows (x86_64)

**Requirements (on macOS):**
```bash
# Install MinGW-w64
brew install mingw-w64

# Add Rust target
rustup target add x86_64-pc-windows-gnu
```

**Build:**
```bash
./scripts/build-windows.sh
```

**Output:**
- EXE: `apps/desktop/src-tauri/target/x86_64-pc-windows-gnu/release/openminedash.exe`

**Deploy to Windows:**
1. Copy `openminedash.exe` to Windows machine
2. Install WebView2 Runtime (if needed)
3. Run the executable

---

## 🔧 Troubleshooting

### macOS

**Error: `xcode-select: error: tool 'xcodebuild' requires Xcode`**
```bash
xcode-select --install
```

**Error: `rustc not found`**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Raspberry Pi 5 Cross-Compilation

**Error: `aarch64-linux-gnu-gcc not found`**
```bash
brew tap messense/macos-cross-toolchains
brew install aarch64-unknown-linux-gnu
```

**Error: `error: linker 'aarch64-linux-gnu-gcc' not found`**
- Ensure the cross-compiler is in your PATH
- Restart terminal after installation

### Windows Cross-Compilation

**Error: `x86_64-w64-mingw32-gcc not found`**
```bash
brew install mingw-w64
```

**Error: `error: linking with 'x86_64-w64-mingw32-gcc' failed`**
- Ensure MinGW is properly installed
- Check `.cargo/config.toml` configuration

---

## 📊 Build Times (Approximate)

| Platform | First Build | Incremental |
|----------|-------------|-------------|
| macOS (M1) | ~5 min | ~30 sec |
| Pi5 (cross) | ~8 min | ~45 sec |
| Windows (cross) | ~6 min | ~35 sec |

---

## 🎯 Build Artifacts

After building, you'll find:

```
mOpenMiner/
├── apps/desktop/src-tauri/target/
│   ├── release/
│   │   └── bundle/
│   │       ├── dmg/              # macOS DMG
│   │       └── macos/            # macOS .app
│   ├── aarch64-unknown-linux-gnu/
│   │   └── release/
│   │       └── openminedash      # Pi5 binary
│   └── x86_64-pc-windows-gnu/
│       └── release/
│           └── openminedash.exe  # Windows binary
```

---

## 🚢 Creating Release Packages

### macOS
```bash
cd apps/desktop/src-tauri/target/release/bundle/dmg/
zip -r OpenMiner-macOS-arm64.zip OpenMiner_*.dmg
```

### Raspberry Pi 5
```bash
cd apps/desktop/src-tauri/target/aarch64-unknown-linux-gnu/release/
tar -czf OpenMiner-pi5-arm64.tar.gz openminedash
```

### Windows
```bash
cd apps/desktop/src-tauri/target/x86_64-pc-windows-gnu/release/
zip OpenMiner-windows-x64.zip openminedash.exe
```

---

## 🤖 Automated Builds (GitHub Actions)

The project includes a GitHub Actions workflow that automatically builds for all platforms:

**Trigger:**
- Push to `main` or `develop` branch
- Create a tag starting with `v` (e.g., `v1.0.0`)
- Manual workflow dispatch

**Workflow file:** `.github/workflows/build-multiplatform.yml`

**To create a release:**
```bash
git tag v1.0.0
git push origin v1.0.0
```

This will automatically:
1. Build for macOS, Pi5, and Windows
2. Create GitHub release
3. Upload all artifacts

---

## 📚 Additional Documentation

- [CROSS_PLATFORM_BUILD.md](docs/CROSS_PLATFORM_BUILD.md) - Detailed cross-platform guide
- [RASPBERRY_PI5.md](docs/RASPBERRY_PI5.md) - Complete Pi5 setup
- [WINDOWS.md](docs/WINDOWS.md) - Complete Windows setup
- [LOCAL_DEVELOPMENT.md](docs/LOCAL_DEVELOPMENT.md) - Development guide

---

## 🆘 Need Help?

- **GitHub Issues**: https://github.com/WeAreTheArtMakers/mOpenMiner/issues
- **Documentation**: See `docs/` folder
- **Community**: [Coming Soon]

---

## 📝 License

MIT License - See [LICENSE](LICENSE)

**Created by WATAM (We Are The Art Makers)**
