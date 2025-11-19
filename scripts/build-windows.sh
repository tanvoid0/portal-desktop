#!/bin/bash
# Build script for Windows (.exe)
# 
# NOTE: This script is for GitHub Actions Windows runners.
# For Windows Docker containers, use build-windows.ps1 instead.

set -e

VERSION=${VERSION:-$(./scripts/get-version.sh)}
RELEASE_TYPE=${RELEASE_TYPE:-release}

echo "Building Windows artifacts..."
echo "Version: $VERSION"
echo "Release Type: $RELEASE_TYPE"

# Ensure we're in the project root
cd "$(dirname "$0")/.." || exit

# Install dependencies
pnpm install --frozen-lockfile

# Build frontend
pnpm build

# Build Tauri app for Windows
# This script is used by GitHub Actions windows-2022 runners
pnpm tauri build --target x86_64-pc-windows-msvc

# Rename artifacts
./scripts/rename-artifacts.sh windows

echo "Windows build complete!"

