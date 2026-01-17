#!/bin/bash
# Ralph installer
# Creates a symlink to ralph in your PATH

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Find the directory where this script lives
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RALPH_BIN="$SCRIPT_DIR/bin/ralph"

# Default install location
INSTALL_DIR="${1:-/usr/local/bin}"

echo ""
echo -e "${BLUE}Ralph Installer${NC}"
echo ""

# Check if ralph binary exists
if [ ! -f "$RALPH_BIN" ]; then
  echo -e "${RED}Error:${NC} bin/ralph not found in $SCRIPT_DIR"
  exit 1
fi

# Check if install directory exists
if [ ! -d "$INSTALL_DIR" ]; then
  echo -e "${YELLOW}Creating directory:${NC} $INSTALL_DIR"
  mkdir -p "$INSTALL_DIR" 2>/dev/null || {
    echo -e "${RED}Error:${NC} Cannot create $INSTALL_DIR"
    echo "Try: sudo ./install.sh"
    exit 1
  }
fi

# Check write permissions
if [ ! -w "$INSTALL_DIR" ]; then
  echo -e "${RED}Error:${NC} Cannot write to $INSTALL_DIR"
  echo ""
  echo "Options:"
  echo "  1. Run with sudo: sudo ./install.sh"
  echo "  2. Install to ~/bin: ./install.sh ~/bin"
  echo ""
  exit 1
fi

# Remove existing symlink or file
TARGET="$INSTALL_DIR/ralph"
if [ -L "$TARGET" ] || [ -f "$TARGET" ]; then
  echo -e "${YELLOW}Removing existing:${NC} $TARGET"
  rm "$TARGET"
fi

# Create symlink
ln -s "$RALPH_BIN" "$TARGET"

echo -e "${GREEN}✓ Installed${NC} ralph -> $RALPH_BIN"
echo ""

# Check if install dir is in PATH
if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  echo -e "${YELLOW}Note:${NC} $INSTALL_DIR is not in your PATH"
  echo ""
  echo "Add to your shell config (~/.bashrc, ~/.zshrc, etc.):"
  echo ""
  echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
  echo ""
fi

echo "Usage:"
echo "  ralph --help      Show help"
echo "  ralph --init      Initialize project with prd.json"
echo "  ralph             Run agent loop in current directory"
echo ""
