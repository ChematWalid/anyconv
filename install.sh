#!/usr/bin/env bash
set -e

REPO="ChematWalid/anyconv"
BIN_NAME="conv"

echo "=== Installing conv (anyconv) ==="

# 1. Detect OS and Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"
TARGET=""

case "$OS" in
    Linux)
        case "$ARCH" in
            x86_64)  TARGET="x86_64-unknown-linux-gnu" ;;
            aarch64) TARGET="aarch64-unknown-linux-gnu" ;;
            *) echo "Unsupported Linux architecture: $ARCH" ;;
        esac
        ;;
    Darwin)
        case "$ARCH" in
            x86_64) TARGET="x86_64-apple-darwin" ;;
            arm64)  TARGET="aarch64-apple-darwin" ;;
            *) echo "Unsupported macOS architecture: $ARCH" ;;
        esac
        ;;
    *)
        echo "Unsupported OS: $OS"
        ;;
esac

BIN_SOURCE=""
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT

# 2. Attempt download from GitHub Releases
if [ -n "$TARGET" ]; then
    RELEASE_URL="https://github.com/${REPO}/releases/latest/download/${BIN_NAME}-${TARGET}.tar.gz"
    echo "Attempting to download prebuilt binary for ${TARGET}..."
    
    if curl -fsSL "$RELEASE_URL" -o "$TMP_DIR/release.tar.gz" 2>/dev/null; then
        tar -xzf "$TMP_DIR/release.tar.gz" -C "$TMP_DIR"
        if [ -f "$TMP_DIR/$BIN_NAME" ]; then
            BIN_SOURCE="$TMP_DIR/$BIN_NAME"
            echo "✓ Downloaded release binary from GitHub."
        fi
    else
        echo "ℹ No prebuilt release binary found at $RELEASE_URL (or repository is private/prerelease)."
    fi
fi

# 3. Fall back to building from source if in repo or cargo is available
if [ -z "$BIN_SOURCE" ]; then
    if [ -f "Cargo.toml" ] && command -v cargo >/dev/null 2>&1; then
        echo "Building from source with cargo..."
        cargo build --release
        BIN_SOURCE="target/release/$BIN_NAME"
    elif command -v cargo >/dev/null 2>&1; then
        echo "Installing with cargo install..."
        cargo install --git "https://github.com/${REPO}.git"
        echo "✓ Installed via cargo to ~/.cargo/bin"
        exit 0
    else
        echo "Error: Could not download prebuilt binary and cargo is not available."
        exit 1
    fi
fi

# 4. Determine installation directory
INSTALL_DIR="/usr/local/bin"
USE_SUDO=false

if [ -w "$INSTALL_DIR" ]; then
    USE_SUDO=false
elif command -v sudo >/dev/null 2>&1; then
    USE_SUDO=true
else
    INSTALL_DIR="$HOME/.local/bin"
    mkdir -p "$INSTALL_DIR"
fi

echo "Installing binary to $INSTALL_DIR..."

if [ "$USE_SUDO" = true ]; then
    sudo cp "$BIN_SOURCE" "$INSTALL_DIR/$BIN_NAME"
    sudo chmod +x "$INSTALL_DIR/$BIN_NAME"
    sudo ln -sf "$INSTALL_DIR/$BIN_NAME" "$INSTALL_DIR/anyconv"
else
    cp "$BIN_SOURCE" "$INSTALL_DIR/$BIN_NAME"
    chmod +x "$INSTALL_DIR/$BIN_NAME"
    ln -sf "$INSTALL_DIR/$BIN_NAME" "$INSTALL_DIR/anyconv"
fi

echo "✓ Installed '$BIN_NAME' and symlink 'anyconv' to $INSTALL_DIR"

# 5. Check /usr/bin/convert conflict / ImageMagick status
if command -v convert >/dev/null 2>&1; then
    CONVERT_PATH=$(command -v convert)
    echo "ℹ Note: '$CONVERT_PATH' is installed on your system (typically ImageMagick)."
    echo "  'conv' and 'anyconv' run independently without touching or overriding 'convert'."
fi

# 6. Generate shell completions
echo "Configuring shell completions..."
mkdir -p "$HOME/.local/share/bash-completion/completions" 2>/dev/null || true
mkdir -p "$HOME/.zsh/completions" 2>/dev/null || true
mkdir -p "$HOME/.zfunc" 2>/dev/null || true
mkdir -p "$HOME/.config/fish/completions" 2>/dev/null || true

"$INSTALL_DIR/$BIN_NAME" completions bash > "$HOME/.local/share/bash-completion/completions/$BIN_NAME" 2>/dev/null || true
"$INSTALL_DIR/$BIN_NAME" completions bash | sed 's/conv/anyconv/g' > "$HOME/.local/share/bash-completion/completions/anyconv" 2>/dev/null || true

"$INSTALL_DIR/$BIN_NAME" completions zsh > "$HOME/.zsh/completions/_$BIN_NAME" 2>/dev/null || true
"$INSTALL_DIR/$BIN_NAME" completions zsh | sed 's/conv/anyconv/g' > "$HOME/.zsh/completions/_anyconv" 2>/dev/null || true
cp "$HOME/.zsh/completions/_$BIN_NAME" "$HOME/.zfunc/_$BIN_NAME" 2>/dev/null || true
cp "$HOME/.zsh/completions/_anyconv" "$HOME/.zfunc/_anyconv" 2>/dev/null || true

"$INSTALL_DIR/$BIN_NAME" completions fish > "$HOME/.config/fish/completions/$BIN_NAME.fish" 2>/dev/null || true
"$INSTALL_DIR/$BIN_NAME" completions fish | sed 's/conv/anyconv/g' > "$HOME/.config/fish/completions/anyconv.fish" 2>/dev/null || true

echo "✓ Shell completions generated (bash, zsh, fish)"

# 7. Configure tldr / tealdeer pages
mkdir -p "$HOME/.local/share/tealdeer/pages" "$HOME/.cache/tealdeer/tldr-pages/pages.en/common" 2>/dev/null || true
if [ -f "docs/conv.md" ]; then
    cp docs/conv.md "$HOME/.local/share/tealdeer/pages/conv.md" 2>/dev/null || true
    cp docs/conv.md "$HOME/.local/share/tealdeer/pages/anyconv.md" 2>/dev/null || true
    cp docs/conv.md "$HOME/.cache/tealdeer/tldr-pages/pages.en/common/conv.md" 2>/dev/null || true
    cp docs/conv.md "$HOME/.cache/tealdeer/tldr-pages/pages.en/common/anyconv.md" 2>/dev/null || true
    echo "✓ tldr / tealdeer pages installed ('tldr conv')"
fi

# 8. Configure navi cheatsheet
mkdir -p "$HOME/.local/share/navi/cheats" 2>/dev/null || true
if [ -f "docs/conv.cheat" ]; then
    cp docs/conv.cheat "$HOME/.local/share/navi/cheats/conv.cheat" 2>/dev/null || true
    echo "✓ navi cheatsheet installed ('navi --query conv')"
fi

# 9. Configure man pages
if [ -f "docs/conv.1" ]; then
    if [ "$USE_SUDO" = true ]; then
        sudo mkdir -p /usr/local/share/man/man1 2>/dev/null || true
        sudo cp docs/conv.1 /usr/local/share/man/man1/conv.1 2>/dev/null || true
        sudo cp docs/conv.1 /usr/local/share/man/man1/anyconv.1 2>/dev/null || true
    else
        mkdir -p "$HOME/.local/share/man/man1" 2>/dev/null || true
        cp docs/conv.1 "$HOME/.local/share/man/man1/conv.1" 2>/dev/null || true
        cp docs/conv.1 "$HOME/.local/share/man/man1/anyconv.1" 2>/dev/null || true
    fi
    echo "✓ man page installed ('man conv')"
fi

echo "=== Installation Complete! Run 'conv --help' or 'conv --list' to get started ==="
