#!/bin/bash
# Build script for Linux (.deb and .AppImage) - Local version

set -e

# Get the script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$PROJECT_ROOT"

VERSION=${VERSION:-$(./scripts/get-version.sh)}
RELEASE_TYPE=${RELEASE_TYPE:-release}

echo "Building Linux artifacts locally..."
echo "Version: $VERSION"
echo "Release Type: $RELEASE_TYPE"
echo "Project root: $PROJECT_ROOT"
echo ""

# Check system dependencies
echo "Checking system dependencies..."
MISSING_DEPS=()

if ! pkg-config --exists webkit2gtk-4.1 2>/dev/null; then
    MISSING_DEPS+=("libwebkit2gtk-4.1-dev")
fi

if ! pkg-config --exists gtk+-3.0 2>/dev/null; then
    MISSING_DEPS+=("libgtk-3-dev")
fi

if ! pkg-config --exists openssl 2>/dev/null; then
    MISSING_DEPS+=("libssl-dev")
fi

if [ ${#MISSING_DEPS[@]} -gt 0 ]; then
    echo "⚠️  Missing dependencies: ${MISSING_DEPS[*]}"
    echo "Install with: sudo apt-get install ${MISSING_DEPS[*]} libayatana-appindicator3-dev librsvg2-dev patchelf"
    read -p "Continue anyway? (y/N) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
fi

# Check Rust target
if ! rustup target list --installed | grep -q x86_64-unknown-linux-gnu; then
    echo "Installing Rust Linux target..."
    rustup target add x86_64-unknown-linux-gnu
fi

# Install dependencies
echo ""
echo "Installing dependencies..."
pnpm install --frozen-lockfile

# Build frontend
echo ""
echo "Building frontend..."
pnpm build

# Build Tauri app for Linux
echo ""
echo "Building Tauri application for Linux..."
echo "This may take several minutes..."
pnpm tauri build --target x86_64-unknown-linux-gnu

# Rename artifacts
echo ""
echo "Renaming artifacts..."
chmod +x scripts/*.sh
mkdir -p build-output
VERSION="$VERSION" \
RELEASE_TYPE="$RELEASE_TYPE" \
./scripts/rename-artifacts.sh linux || {
    echo "⚠️  Artifact renaming failed, but build may have succeeded"
    echo "Check: src-tauri/target/release/bundle/"
}

echo ""
echo "✅ Linux build complete!"
echo ""
echo "Artifacts location:"
echo "  - build-output/ (renamed artifacts)"
echo "  - src-tauri/target/release/bundle/ (original Tauri output)"
echo ""
if [ -d "build-output" ]; then
    ls -lh build-output/ 2>/dev/null || echo "No files in build-output/"
fi

