#!/bin/bash
# Build script for all platforms
# 
# NOTE: This script is for GitHub Actions or local builds on native hosts.
# Windows and macOS builds require their respective native hosts.
# For Docker, only Linux builds are fully supported.

set -e

VERSION=${VERSION:-$(./scripts/get-version.sh)}
RELEASE_TYPE=${RELEASE_TYPE:-release}

echo "Building for all platforms..."
echo "Version: $VERSION"
echo "Release Type: $RELEASE_TYPE"
echo ""
echo "⚠️  Note: Windows and macOS builds require native hosts"
echo "   - Windows: Requires Windows host or GitHub Actions"
echo "   - macOS: Requires macOS host or GitHub Actions"
echo "   - Linux: Can be built in Docker"
echo ""

# Build Linux (works in Docker)
./scripts/build-linux.sh

# Build Windows (requires Windows host or GitHub Actions)
if [[ "$OSTYPE" == "msys" || "$OSTYPE" == "win32" ]] || [ -n "$GITHUB_ACTIONS" ]; then
    ./scripts/build-windows.sh
else
    echo "⚠️  Skipping Windows build - requires Windows host or GitHub Actions"
fi

# Build macOS (requires macOS host or GitHub Actions)
if [[ "$OSTYPE" == "darwin"* ]] || [ -n "$GITHUB_ACTIONS" ]; then
    ./scripts/build-macos.sh
else
    echo "⚠️  Skipping macOS build - requires macOS host or GitHub Actions"
fi

echo ""
echo "All platform builds complete!"
echo ""
echo "Artifacts in build-output/:"
ls -lh build-output/ 2>/dev/null || echo "No artifacts found"

