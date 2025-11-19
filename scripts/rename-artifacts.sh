#!/bin/bash
# Rename build artifacts with version and release type

set -e

VERSION=${VERSION:-$(./scripts/get-version.sh)}
RELEASE_TYPE=${RELEASE_TYPE:-release}
PRODUCT_NAME="portal-desktop"

# Create output directory
mkdir -p build-output

# Function to rename Linux artifacts
rename_linux() {
    local bundle_dir="src-tauri/target/release/bundle"
    
    if [ -d "$bundle_dir/deb" ]; then
        find "$bundle_dir/deb" -name "*.deb" -type f | while read deb_file; do
            filename=$(basename "$deb_file")
            new_name="${PRODUCT_NAME}-${VERSION}-${RELEASE_TYPE}-linux.deb"
            cp "$deb_file" "build-output/$new_name"
            echo "Created: build-output/$new_name"
        done
    fi
    
    if [ -d "$bundle_dir/appimage" ]; then
        find "$bundle_dir/appimage" -name "*.AppImage" -type f | while read appimage_file; do
            filename=$(basename "$appimage_file")
            new_name="${PRODUCT_NAME}-${VERSION}-${RELEASE_TYPE}-linux.AppImage"
            cp "$appimage_file" "build-output/$new_name"
            chmod +x "build-output/$new_name"
            echo "Created: build-output/$new_name"
        done
    fi
}

# Function to rename Windows artifacts
rename_windows() {
    local bundle_dir="src-tauri/target/release/bundle"
    
    if [ -d "$bundle_dir/nsis" ]; then
        find "$bundle_dir/nsis" -name "*.exe" -type f | while read exe_file; do
            filename=$(basename "$exe_file")
            new_name="${PRODUCT_NAME}-${VERSION}-${RELEASE_TYPE}-windows.exe"
            cp "$exe_file" "build-output/$new_name"
            echo "Created: build-output/$new_name"
        done
    fi
    
    if [ -d "$bundle_dir/msi" ]; then
        find "$bundle_dir/msi" -name "*.msi" -type f | while read msi_file; do
            filename=$(basename "$msi_file")
            new_name="${PRODUCT_NAME}-${VERSION}-${RELEASE_TYPE}-windows.msi"
            cp "$msi_file" "build-output/$new_name"
            echo "Created: build-output/$new_name"
        done
    fi
}

# Function to rename macOS artifacts
rename_macos() {
    local bundle_dir="src-tauri/target/release/bundle"
    
    if [ -d "$bundle_dir/dmg" ]; then
        find "$bundle_dir/dmg" -name "*.dmg" -type f | while read dmg_file; do
            filename=$(basename "$dmg_file")
            new_name="${PRODUCT_NAME}-${VERSION}-${RELEASE_TYPE}-macos.dmg"
            cp "$dmg_file" "build-output/$new_name"
            echo "Created: build-output/$new_name"
        done
    fi
    
    if [ -d "$bundle_dir/macos" ]; then
        find "$bundle_dir/macos" -name "*.app" -type d | while read app_dir; do
            app_name=$(basename "$app_dir")
            new_name="${PRODUCT_NAME}-${VERSION}-${RELEASE_TYPE}-macos.app"
            # Create a zip of the .app bundle
            cd "$(dirname "$app_dir")"
            zip -r "../../../../build-output/${PRODUCT_NAME}-${VERSION}-${RELEASE_TYPE}-macos.zip" "$app_name"
            cd - > /dev/null
            echo "Created: build-output/${PRODUCT_NAME}-${VERSION}-${RELEASE_TYPE}-macos.zip"
        done
    fi
}

# Main execution
PLATFORM=${1:-all}

case "$PLATFORM" in
    linux)
        rename_linux
        ;;
    windows)
        rename_windows
        ;;
    macos)
        rename_macos
        ;;
    all)
        rename_linux
        rename_windows
        rename_macos
        ;;
    *)
        echo "Unknown platform: $PLATFORM"
        exit 1
        ;;
esac

echo ""
echo "All artifacts renamed and copied to build-output/"
ls -lh build-output/

