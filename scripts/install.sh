#!/usr/bin/env sh
set -eu

REPOSITORY="${FPT_INSTALL_REPOSITORY:-dcc-mcp/fpt-cli}"
INSTALL_DIR="${FPT_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${1:-${FPT_INSTALL_VERSION:-latest}}"

require_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        echo "error: required command '$1' is not available" >&2
        exit 1
    fi
}

require_command curl
require_command tar
require_command install

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64|amd64)
                TARGET="x86_64-unknown-linux-gnu"
                ;;
            *)
                echo "error: unsupported Linux architecture: $ARCH" >&2
                exit 1
                ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64)
                TARGET="x86_64-apple-darwin"
                ;;
            arm64|aarch64)
                TARGET="aarch64-apple-darwin"
                ;;
            *)
                echo "error: unsupported macOS architecture: $ARCH" >&2
                exit 1
                ;;
        esac
        ;;
    *)
        echo "error: unsupported operating system: $OS" >&2
        exit 1
        ;;
esac

ASSET="fpt-${TARGET}.tar.gz"
case "$VERSION" in
    latest|'')
        DOWNLOAD_ASSET="$ASSET"
        DOWNLOAD_URL="https://github.com/${REPOSITORY}/releases/latest/download/${DOWNLOAD_ASSET}"
        ;;
    v*)
        DOWNLOAD_ASSET="fpt-${VERSION}-${TARGET}.tar.gz"
        DOWNLOAD_URL="https://github.com/${REPOSITORY}/releases/download/${VERSION}/${DOWNLOAD_ASSET}"
        ;;
    *)
        DOWNLOAD_ASSET="fpt-v${VERSION}-${TARGET}.tar.gz"
        DOWNLOAD_URL="https://github.com/${REPOSITORY}/releases/download/v${VERSION}/${DOWNLOAD_ASSET}"
        ;;
esac

TMP_DIR="$(mktemp -d 2>/dev/null || mktemp -d -t fpt-install)"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT INT TERM

ARCHIVE_PATH="$TMP_DIR/$DOWNLOAD_ASSET"
mkdir -p "$INSTALL_DIR"

echo "Downloading ${DOWNLOAD_ASSET} from ${DOWNLOAD_URL}"
curl -fsSL "$DOWNLOAD_URL" -o "$ARCHIVE_PATH"
tar -xzf "$ARCHIVE_PATH" -C "$TMP_DIR"
install -m 755 "$TMP_DIR/fpt" "$INSTALL_DIR/fpt"

echo "Installed fpt to $INSTALL_DIR/fpt"
case ":$PATH:" in
    *":$INSTALL_DIR:"*) ;;
    *) echo "Add $INSTALL_DIR to PATH to run 'fpt' directly." ;;
esac
