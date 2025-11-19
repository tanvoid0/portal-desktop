#!/bin/bash
# Build script for Linux (.deb and .AppImage)

set -e

VERSION=${VERSION:-$(./scripts/get-version.sh)}
RELEASE_TYPE=${RELEASE_TYPE:-release}

echo "Building Linux artifacts..."
echo "Version: $VERSION"
echo "Release Type: $RELEASE_TYPE"

# Ensure we're in the project root
cd /app

# Install dependencies
pnpm install --frozen-lockfile

# Build frontend
pnpm build

# Build Tauri app for Linux
# Tauri will generate both .deb and .AppImage by default
pnpm tauri build --target x86_64-unknown-linux-gnu

# Rename artifacts
./scripts/rename-artifacts.sh linux

echo "Linux build complete!"

