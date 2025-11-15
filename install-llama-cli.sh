#!/bin/bash
# Install FastForth Llama CLI to user's PATH

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLI_SOURCE="$SCRIPT_DIR/bin/fastforth-llama"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo "FastForth Llama CLI Installer"
echo "=============================="
echo ""

# Check if source exists
if [ ! -f "$CLI_SOURCE" ]; then
    echo -e "${RED}Error: $CLI_SOURCE not found${NC}"
    exit 1
fi

# Determine installation directory
if [ -d "$HOME/bin" ] && [[ ":$PATH:" == *":$HOME/bin:"* ]]; then
    INSTALL_DIR="$HOME/bin"
elif [ -d "$HOME/.local/bin" ] && [[ ":$PATH:" == *":$HOME/.local/bin:"* ]]; then
    INSTALL_DIR="$HOME/.local/bin"
elif [ -d "/usr/local/bin" ] && [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
else
    echo -e "${YELLOW}No suitable directory found in PATH${NC}"
    echo "Please choose installation directory:"
    echo "  1) $HOME/bin (will be created)"
    echo "  2) $HOME/.local/bin (will be created)"
    echo "  3) /usr/local/bin (requires sudo)"
    echo "  4) Custom path"
    read -p "Choice [1-4]: " choice

    case $choice in
        1)
            INSTALL_DIR="$HOME/bin"
            mkdir -p "$INSTALL_DIR"
            echo ""
            echo -e "${YELLOW}Note: Add this to your ~/.bashrc or ~/.zshrc:${NC}"
            echo "  export PATH=\"\$HOME/bin:\$PATH\""
            ;;
        2)
            INSTALL_DIR="$HOME/.local/bin"
            mkdir -p "$INSTALL_DIR"
            echo ""
            echo -e "${YELLOW}Note: Add this to your ~/.bashrc or ~/.zshrc:${NC}"
            echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
            ;;
        3)
            INSTALL_DIR="/usr/local/bin"
            ;;
        4)
            read -p "Enter installation directory: " INSTALL_DIR
            mkdir -p "$INSTALL_DIR"
            ;;
        *)
            echo -e "${RED}Invalid choice${NC}"
            exit 1
            ;;
    esac
fi

# Install
INSTALL_PATH="$INSTALL_DIR/fastforth-llama"

echo ""
echo "Installing to: $INSTALL_PATH"

if [ -f "$INSTALL_PATH" ]; then
    echo -e "${YELLOW}Warning: $INSTALL_PATH already exists${NC}"
    read -p "Overwrite? [y/N] " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Installation cancelled"
        exit 1
    fi
fi

# Copy and make executable
cp "$CLI_SOURCE" "$INSTALL_PATH"
chmod +x "$INSTALL_PATH"

echo -e "${GREEN}✓ Installed successfully${NC}"
echo ""

# Verify installation
if command -v fastforth-llama &> /dev/null; then
    echo -e "${GREEN}✓ fastforth-llama is now in your PATH${NC}"
    echo ""
    echo "Try it:"
    echo "  fastforth-llama \"What is recursion?\""
    echo "  fastforth-llama -i  # Interactive mode"
else
    echo -e "${YELLOW}⚠ Installation complete, but not in current PATH${NC}"
    echo ""
    echo "You may need to:"
    echo "  1. Restart your shell"
    echo "  2. Or run: export PATH=\"$INSTALL_DIR:\$PATH\""
fi

echo ""
echo "Optional: Install dependencies"
echo "  brew install jq  # Better JSON parsing (macOS)"
echo "  sudo apt-get install jq  # Better JSON parsing (Linux)"
echo ""
echo "Make sure Ollama is running:"
echo "  ollama serve"
echo "  ollama pull llama3.2"
