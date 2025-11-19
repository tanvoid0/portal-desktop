#!/bin/bash
# Build script for macOS (.dmg)

set -e

VERSION=${VERSION:-$(./scripts/get-version.sh)}
RELEASE_TYPE=${RELEASE_TYPE:-release}

echo "Building macOS artifacts..."
echo "Version: $VERSION"
echo "Release Type: $RELEASE_TYPE"

# Ensure we're in the project root
cd /app

# Install dependencies
pnpm install --frozen-lockfile

# Build frontend
pnpm build

# Build Tauri app for macOS
# Note: macOS builds typically require macOS host
# For production, use GitHub Actions macos-latest runners
# This script is for reference - actual builds should use native macOS
pnpm tauri build --target x86_64-apple-darwin
pnpm tauri build --target aarch64-apple-darwin

# Rename artifacts
./scripts/rename-artifacts.sh macos

echo "macOS build complete!"

