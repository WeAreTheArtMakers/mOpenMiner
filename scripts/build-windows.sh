#!/bin/bash
# Build script for Windows (x86_64)

set -e

echo "🔧 Building OpenMiner for Windows (x86_64)..."

# Check if cross-compilation tools are installed
if ! command -v x86_64-w64-mingw32-gcc &> /dev/null; then
    echo "❌ x86_64-w64-mingw32-gcc not found!"
    echo "📦 Installing cross-compilation tools..."
    
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        if ! command -v brew &> /dev/null; then
            echo "❌ Homebrew not found. Please install Homebrew first."
            exit 1
        fi
        
        echo "Installing MinGW-w64 cross-compiler via Homebrew..."
        brew install mingw-w64
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        sudo apt-get update
        sudo apt-get install -y mingw-w64
    fi
fi

# Add Rust target for Windows
echo "📦 Adding Rust target for Windows..."
rustup target add x86_64-pc-windows-gnu

# Build frontend
echo "🎨 Building frontend..."
pnpm install
pnpm build

# Build Tauri app for Windows
echo "🦀 Building Rust backend for Windows..."
cd apps/desktop/src-tauri

cargo build --release --target x86_64-pc-windows-gnu

cd ../../..

echo "✅ Build complete!"
echo "📦 Binary location: apps/desktop/src-tauri/target/x86_64-pc-windows-gnu/release/openminedash.exe"
echo ""
echo "📋 To distribute for Windows:"
echo "   1. Copy the .exe file to a Windows machine"
echo "   2. Include WebView2 runtime (or let users install it)"
echo "   3. Package with assets/coins folder"
