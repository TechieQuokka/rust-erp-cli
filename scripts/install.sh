#!/bin/bash
set -e

# ERP CLI Installation Script
# Usage: curl -fsSL https://raw.githubusercontent.com/example/erp-cli/main/scripts/install.sh | bash

REPO="example/erp-cli"
BINARY_NAME="erp"
INSTALL_DIR="/usr/local/bin"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Logging functions
log() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

error() {
    echo -e "${RED}[ERROR]${NC} $1"
    exit 1
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect platform and architecture
detect_platform() {
    local platform=$(uname -s | tr '[:upper:]' '[:lower:]')
    local arch=$(uname -m)

    case $platform in
        linux*)
            PLATFORM="linux"
            ;;
        darwin*)
            PLATFORM="macos"
            ;;
        mingw*|msys*|cygwin*)
            PLATFORM="windows"
            ;;
        *)
            error "Unsupported platform: $platform"
            ;;
    esac

    case $arch in
        x86_64|amd64)
            ARCH="amd64"
            ;;
        aarch64|arm64)
            ARCH="arm64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            ;;
    esac

    if [[ "$PLATFORM" == "windows" ]]; then
        BINARY_NAME="erp.exe"
        ASSET_NAME="erp-windows-${ARCH}.exe"
    else
        ASSET_NAME="erp-${PLATFORM}-${ARCH}"
    fi

    log "Detected platform: $PLATFORM"
    log "Detected architecture: $ARCH"
}

# Get latest release version
get_latest_version() {
    if command_exists curl; then
        VERSION=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command_exists wget; then
        VERSION=$(wget -qO- "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        error "Neither curl nor wget is available. Please install one of them."
    fi

    if [[ -z "$VERSION" ]]; then
        error "Failed to get latest version"
    fi

    log "Latest version: $VERSION"
}

# Download binary
download_binary() {
    local download_url="https://github.com/$REPO/releases/download/$VERSION/$ASSET_NAME"
    local temp_file="/tmp/$BINARY_NAME"

    log "Downloading from: $download_url"

    if command_exists curl; then
        curl -L "$download_url" -o "$temp_file"
    elif command_exists wget; then
        wget -O "$temp_file" "$download_url"
    else
        error "Neither curl nor wget is available"
    fi

    if [[ ! -f "$temp_file" ]]; then
        error "Download failed"
    fi

    log "Downloaded to: $temp_file"
}

# Install binary
install_binary() {
    local temp_file="/tmp/$BINARY_NAME"
    local install_path="$INSTALL_DIR/$BINARY_NAME"

    # Check if we need sudo
    if [[ ! -w "$INSTALL_DIR" ]]; then
        if command_exists sudo; then
            log "Installing to $install_path (requires sudo)"
            sudo mv "$temp_file" "$install_path"
            sudo chmod +x "$install_path"
        else
            error "Cannot write to $INSTALL_DIR and sudo is not available"
        fi
    else
        log "Installing to $install_path"
        mv "$temp_file" "$install_path"
        chmod +x "$install_path"
    fi

    log "Successfully installed $BINARY_NAME to $install_path"
}

# Verify installation
verify_installation() {
    if command_exists "$BINARY_NAME"; then
        log "✅ Installation verified!"
        log "Run '$BINARY_NAME --help' to get started"
        "$BINARY_NAME" --version
    else
        warn "Installation completed but $BINARY_NAME is not in PATH"
        warn "You may need to add $INSTALL_DIR to your PATH"
        warn "Or restart your shell"
    fi
}

# Main installation process
main() {
    log "Starting ERP CLI installation..."

    # Check prerequisites
    if ! command_exists curl && ! command_exists wget; then
        error "Either curl or wget is required for installation"
    fi

    detect_platform
    get_latest_version
    download_binary
    install_binary
    verify_installation

    log "🎉 Installation complete!"
    log ""
    log "Next steps:"
    log "1. Initialize database: $BINARY_NAME migrate init"
    log "2. Add your first product: $BINARY_NAME inventory add 'Product Name' --sku 'SKU001' --quantity 10 --price 29.99"
    log "3. View all commands: $BINARY_NAME --help"
}

# Handle script arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --version)
            VERSION="$2"
            shift 2
            ;;
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --help)
            echo "ERP CLI Installation Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --version VERSION    Install specific version (default: latest)"
            echo "  --install-dir DIR    Install directory (default: /usr/local/bin)"
            echo "  --help              Show this help message"
            echo ""
            echo "Example:"
            echo "  $0 --version v1.0.0 --install-dir ~/.local/bin"
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            ;;
    esac
done

# Run main function
main