#!/bin/bash
# Build script for Raspberry Pi 5 (ARM64 Linux)

set -e

echo "🔧 Building OpenMiner for Raspberry Pi 5 (ARM64 Linux)..."

# Check if cross-compilation tools are installed
if ! command -v aarch64-linux-gnu-gcc &> /dev/null; then
    echo "❌ aarch64-linux-gnu-gcc not found!"
    echo "📦 Installing cross-compilation tools..."
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        if ! command -v brew &> /dev/null; then
            echo "❌ Homebrew not found. Please install Homebrew first."
            exit 1
        fi
        
        echo "Installing ARM64 cross-compiler via Homebrew..."
        brew tap messense/macos-cross-toolchains
        brew install aarch64-unknown-linux-gnu
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        sudo apt-get update
        sudo apt-get install -y gcc-aarch64-linux-gnu g++-aarch64-linux-gnu
    fi
fi

# Add Rust target for ARM64 Linux
echo "📦 Adding Rust target for ARM64 Linux..."
rustup target add aarch64-unknown-linux-gnu

# Install required system dependencies for Tauri on Linux
echo "📦 Installing Tauri dependencies..."
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    sudo apt-get install -y \
        libwebkit2gtk-4.0-dev \
        build-essential \
        curl \
        wget \
        file \
        libssl-dev \
        libgtk-3-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev
fi

# Build frontend
echo "🎨 Building frontend..."
pnpm install
pnpm build

# Build Tauri app for ARM64 Linux
echo "🦀 Building Rust backend for ARM64 Linux..."
cd apps/desktop/src-tauri

# Set environment variables for cross-compilation
export PKG_CONFIG_SYSROOT_DIR=/usr/aarch64-linux-gnu
export PKG_CONFIG_PATH=/usr/aarch64-linux-gnu/lib/pkgconfig

cargo build --release --target aarch64-unknown-linux-gnu

cd ../../..

echo "✅ Build complete!"
echo "📦 Binary location: apps/desktop/src-tauri/target/aarch64-unknown-linux-gnu/release/openminedash"
echo ""
echo "📋 To create a distributable package for Raspberry Pi 5:"
echo "   1. Copy the binary to your Pi 5"
echo "   2. Install dependencies on Pi: sudo apt-get install libwebkit2gtk-4.0-37 libayatana-appindicator3-1"
echo "   3. Run: chmod +x openminedash && ./openminedash"
