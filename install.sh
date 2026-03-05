#!/bin/sh
# christ-cli installer
# Usage: curl -fsSL https://raw.githubusercontent.com/whoisyurii/christ-cli/main/install.sh | sh

set -e

REPO="whoisyurii/christ-cli"
BINARY="christ"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux)  OS="unknown-linux-gnu" ;;
        Darwin) OS="apple-darwin" ;;
        *)      echo "Error: Unsupported OS: $OS"; exit 1 ;;
    esac

    case "$ARCH" in
        x86_64|amd64)  ARCH="x86_64" ;;
        arm64|aarch64) ARCH="aarch64" ;;
        *)             echo "Error: Unsupported architecture: $ARCH"; exit 1 ;;
    esac

    TARGET="${ARCH}-${OS}"
}

# Get latest release tag from GitHub
get_latest_version() {
    VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')
    if [ -z "$VERSION" ]; then
        echo "Error: Could not determine latest version"
        exit 1
    fi
}

main() {
    echo ""
    echo "  Installing christ-cli..."
    echo ""

    detect_platform
    get_latest_version

    URL="https://github.com/${REPO}/releases/download/v${VERSION}/${BINARY}-${TARGET}.tar.gz"

    echo "  Platform:  ${TARGET}"
    echo "  Version:   v${VERSION}"
    echo "  URL:       ${URL}"
    echo ""

    # Download and extract
    TMPDIR=$(mktemp -d)
    trap 'rm -rf "$TMPDIR"' EXIT

    echo "  Downloading..."
    curl -fsSL "$URL" -o "${TMPDIR}/christ.tar.gz"

    echo "  Extracting..."
    tar xzf "${TMPDIR}/christ.tar.gz" -C "$TMPDIR"

    echo "  Installing to ${INSTALL_DIR}..."
    if [ -w "$INSTALL_DIR" ]; then
        mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    else
        sudo mv "${TMPDIR}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    fi
    chmod +x "${INSTALL_DIR}/${BINARY}"

    echo ""
    echo "  christ-cli v${VERSION} installed successfully!"
    echo ""
    echo "  Run 'christ' to start reading the Bible."
    echo ""
}

main
