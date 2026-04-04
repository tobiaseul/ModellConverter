#!/bin/bash

# ModellConverter Build All Script
# Builds CLI and GUI for the current platform

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Show help if requested
if [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: ./build-all.sh"
    echo ""
    echo "Builds ModellConverter CLI and GUI for the current platform."
    echo ""
    echo "Requirements:"
    echo "  - Rust 1.70+ (https://www.rust-lang.org/tools/install)"
    echo "  - Platform-specific build tools (see README.md)"
    exit 0
fi

echo "🔨 ModellConverter Build All"
echo "======================================"

# Detect current platform
PLATFORM=$(uname -s)
ARCH=$(uname -m)

echo -e "${BLUE}Detected: $PLATFORM $ARCH${NC}"
echo ""

# Build CLI
echo -e "${BLUE}Building CLI...${NC}"
cargo build --release
echo -e "${GREEN}✓ CLI built${NC}"
echo ""

# Build GUI (Tauri)
echo -e "${BLUE}Building GUI (Tauri)...${NC}"
cd src-tauri

if [ "$PLATFORM" = "Darwin" ]; then
    echo -e "${YELLOW}Building for macOS...${NC}"
    cargo tauri build
    echo -e "${GREEN}✓ macOS build complete${NC}"

elif [ "$PLATFORM" = "Linux" ]; then
    echo -e "${YELLOW}Building for Linux...${NC}"
    cargo tauri build
    echo -e "${GREEN}✓ Linux builds complete (AppImage + deb)${NC}"

elif [ "$PLATFORM" = "MINGW64_NT" ] || [ "$PLATFORM" = "CYGWIN_NT-10.0" ] || [[ "$PLATFORM" == "MSYS_NT"* ]]; then
    echo -e "${YELLOW}Building for Windows...${NC}"
    cargo tauri build
    echo -e "${GREEN}✓ Windows build complete${NC}"

else
    echo -e "${YELLOW}Unknown platform: $PLATFORM${NC}"
    echo "Building for current platform..."
    cargo tauri build
fi

cd ..

# Copy artifacts to build/ folder
echo ""
echo -e "${BLUE}Collecting artifacts...${NC}"

# Clean up old build folder
if [ -d "build" ]; then
    echo "  → Removing old build/ folder"
    rm -rf build
fi

# Create build folder structure
mkdir -p build/cli build/gui
echo -e "${GREEN}✓ Created build/ folder${NC}"

# Copy CLI binary (Unix or Windows)
if [ -f "target/release/modell-converter" ]; then
    cp target/release/modell-converter build/cli/
    echo -e "${GREEN}✓ Copied CLI: build/cli/modell-converter${NC}"
elif [ -f "target/release/modell-converter.exe" ]; then
    cp target/release/modell-converter.exe build/cli/
    echo -e "${GREEN}✓ Copied CLI: build/cli/modell-converter.exe${NC}"
else
    echo -e "${YELLOW}⊘ CLI binary not found${NC}"
fi

# Copy GUI artifacts (all types)
# Search in target/*/release/bundle and target/release/bundle
echo "  → Searching for artifacts in target/ directories..."

find target -path "*/release/bundle" -type d 2>/dev/null | while read bundle_dir; do
    # macOS DMG files
    find "$bundle_dir/dmg" -type f -name "*.dmg" 2>/dev/null | while read file; do
        cp "$file" build/gui/
        echo -e "${GREEN}✓ Copied: $(basename "$file")${NC}"
    done

    # macOS APP bundles
    find "$bundle_dir/macos" -type d -name "*.app" 2>/dev/null | while read app; do
        cp -r "$app" build/gui/
        echo -e "${GREEN}✓ Copied: $(basename "$app")${NC}"
    done

    # Linux DEB packages
    find "$bundle_dir/deb" -type f -name "*.deb" 2>/dev/null | while read file; do
        cp "$file" build/gui/
        echo -e "${GREEN}✓ Copied: $(basename "$file")${NC}"
    done

    # Linux AppImage
    find "$bundle_dir/appimage" -type f -name "*.AppImage" 2>/dev/null | while read file; do
        cp "$file" build/gui/
        echo -e "${GREEN}✓ Copied: $(basename "$file")${NC}"
    done

    # Windows EXE (NSIS)
    find "$bundle_dir/nsis" -type f -name "*.exe" 2>/dev/null | while read file; do
        cp "$file" build/gui/
        echo -e "${GREEN}✓ Copied: $(basename "$file")${NC}"
    done

    # Windows MSI
    find "$bundle_dir/msi" -type f -name "*.msi" 2>/dev/null | while read file; do
        cp "$file" build/gui/
        echo -e "${GREEN}✓ Copied: $(basename "$file")${NC}"
    done
done

# Check if GUI folder has artifacts
if [ -z "$(find build/gui -type f 2>/dev/null)" ]; then
    echo -e "${YELLOW}⊘ No GUI artifacts found${NC}"
fi

echo ""
echo "======================================"
echo -e "${GREEN}✓ All builds complete!${NC}"
echo ""
echo "Artifacts collected in build/ folder:"
echo "  build/cli/          - CLI binary"
echo "  build/gui/          - GUI installers & bundles"
echo ""
echo "Build folder contents:"
if [ -d "build" ]; then
    find build -type f | sed 's|^|  |'
else
    echo "  (empty)"
fi

echo ""
echo "For more information:"
echo "  ./build-all.sh --help"
