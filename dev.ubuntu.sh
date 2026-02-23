#!/bin/bash
# dev.ubuntu.sh - Setup development environment for CC Switch on Ubuntu/Debian
# This script installs all system dependencies required for Tauri development

set -e

echo "=== CC Switch Development Setup for Ubuntu/Debian ==="
echo ""

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Check if running as root
if [ "$EUID" -eq 0 ]; then
   echo -e "${RED}Error: Please do not run this script as root/sudo${NC}"
   echo "The script will ask for sudo password when needed."
   exit 1
fi

# Check if we're on Ubuntu/Debian
if ! command -v apt-get &> /dev/null; then
    echo -e "${RED}Error: This script is designed for Ubuntu/Debian systems with apt-get${NC}"
    exit 1
fi

echo "Step 1: Updating package lists..."
sudo apt-get update

echo ""
echo "Step 2: Installing Tauri system dependencies..."
sudo apt-get install -y \
    libgtk-3-dev \
    libgdk-pixbuf2.0-dev \
    libwebkit2gtk-4.1-dev \
    librsvg2-dev \
    libayatana-appindicator3-dev \
    patchelf
    
echo ""
echo "Step 3: Installing build essentials..."
sudo apt-get install -y \
    build-essential \
    curl \
    wget \
    git \
    pkg-config

echo ""
echo "Step 4: Checking for Rust..."
if command -v rustc &> /dev/null; then
    echo -e "${GREEN}✓ Rust is already installed${NC}"
    rustc --version
else
    echo -e "${YELLOW}⚠ Rust not found. Installing...${NC}"
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo -e "${GREEN}✓ Rust installed successfully${NC}"
    rustc --version
fi

echo ""
echo "Step 5: Checking for Node.js..."
if command -v node &> /dev/null; then
    NODE_VERSION=$(node --version | cut -d'v' -f2 | cut -d'.' -f1)
    if [ "$NODE_VERSION" -ge 18 ]; then
        echo -e "${GREEN}✓ Node.js is already installed (>= 18)${NC}"
        node --version
    else
        echo -e "${YELLOW}⚠ Node.js version is too old. Please upgrade to 18+${NC}"
        echo "Visit: https://nodejs.org/ or use nvm to upgrade"
    fi
else
    echo -e "${YELLOW}⚠ Node.js not found. Installing...${NC}"
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
    sudo apt-get install -y nodejs
    echo -e "${GREEN}✓ Node.js installed successfully${NC}"
    node --version
fi

echo ""
echo "Step 6: Checking for pnpm..."
if command -v pnpm &> /dev/null; then
    echo -e "${GREEN}✓ pnpm is already installed${NC}"
    pnpm --version
else
    echo -e "${YELLOW}⚠ pnpm not found. Installing...${NC}"
    curl -fsSL https://get.pnpm.io/install.sh | sh -
    export PATH="$HOME/.local/share/pnpm:$PATH"
    echo -e "${GREEN}✓ pnpm installed successfully${NC}"
    pnpm --version
fi

echo ""
echo "Step 7: Installing project dependencies..."
if [ -f "package.json" ]; then
    pnpm install
    echo -e "${GREEN}✓ Node dependencies installed${NC}"
else
    echo -e "${YELLOW}⚠ No package.json found. Skipping pnpm install${NC}"
fi

echo ""
echo "Step 8: Checking for Linuxbrew pkg-config conflict..."
if which pkg-config | grep -q "linuxbrew"; then
    echo -e "${YELLOW}Linuxbrew pkg-config detected${NC}"
    echo "Linuxbrew's pkg-config doesn't search system paths by default."
    echo ""
    echo "To fix this, add the following to your ~/.bashrc or ~/.zshrc:"
    echo ""
    echo '  export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:$PKG_CONFIG_PATH'
    echo ""
    echo "Or run with:"
    echo '  PKG_CONFIG=/usr/bin/pkg-config npm run dev'
    echo ""
    export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:$PKG_CONFIG_PATH
    echo -e "${GREEN}Added system pkgconfig paths to current session${NC}"
else
    echo -e "${GREEN}System pkg-config is being used${NC}"
    # Ensure system pkgconfig paths are included even with system pkg-config
    export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/lib/pkgconfig:/usr/share/pkgconfig:$PKG_CONFIG_PATH
fi

echo ""
echo "=== Setup Complete! ==="
echo ""
echo "You can now run the development server with:"
echo ""
echo "  pnpm run dev             # Full desktop app with Tauri"
echo "  pnpm run dev:renderer    # Web-only mode (no desktop features)"
echo "  pnpm run build:linux     # Build Linux distribution packages (.deb, .rpm, .AppImage)"
echo "  pnpm run headless:web    # Start headless web server mode"
echo ""
echo "If you encounter any issues:"
echo "  1. Make sure to restart your terminal or run: source ~/.cargo/env"
echo "  2. Check Tauri prerequisites: https://tauri.app/start/prerequisites/"
echo ""
