#!/bin/bash
# Extract version from package.json or tauri.conf.json
# Priority: tauri.conf.json > package.json

set -e

TAURI_VERSION=$(node -p "require('./src-tauri/tauri.conf.json').version" 2>/dev/null || echo "")
PACKAGE_VERSION=$(node -p "require('./package.json').version" 2>/dev/null || echo "")

if [ -n "$TAURI_VERSION" ]; then
    echo "$TAURI_VERSION"
elif [ -n "$PACKAGE_VERSION" ]; then
    echo "$PACKAGE_VERSION"
else
    echo "0.0.0"
fi

