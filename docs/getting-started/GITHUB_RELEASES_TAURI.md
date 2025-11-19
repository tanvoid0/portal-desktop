# GitHub Releases for Tauri Projects (Windows & macOS)

This guide explains how to set up automated GitHub Releases for Tauri applications with Windows and macOS builds.

## Overview

The standard pattern for Tauri multi-platform releases:

1. **Build on native runners** - Each platform builds on its native runner
2. **Upload artifacts** - Each build job uploads artifacts
3. **Create release** - A final job downloads all artifacts and creates a GitHub Release

## Architecture Pattern

```
┌─────────────┐
│   Prepare   │  Extract version, determine release type
└──────┬──────┘
       │
       ├───► ┌─────────────┐
       │     │ Build Linux │  Ubuntu runner → .deb, .AppImage
       │     └──────┬──────┘
       │            │
       ├───► ┌─────────────┐
       │     │Build Windows│  Windows runner → .exe, .msi
       │     └──────┬──────┘
       │            │
       └───► ┌─────────────┐
             │ Build macOS │  macOS runner → .dmg, .app
             └──────┬──────┘
                    │
                    ▼
            ┌───────────────┐
            │Create Release │  Download all artifacts → GitHub Release
            └───────────────┘
```

## Workflow Structure

### Option 1: Manual Artifact Management (Recommended)

This approach gives you full control over artifact naming and release creation.

```yaml
name: Build and Release

on:
  push:
    branches: [main]
    tags: ['v*.*.*']
  workflow_dispatch:

jobs:
  prepare:
    runs-on: ubuntu-latest
    outputs:
      version: ${{ steps.version.outputs.version }}
      release_type: ${{ steps.release_type.outputs.type }}
    steps:
      - uses: actions/checkout@v4
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '22'
      - name: Extract version
        id: version
        run: |
          VERSION=$(node -p "require('./src-tauri/tauri.conf.json').version")
          echo "version=$VERSION" >> $GITHUB_OUTPUT

  build-linux:
    needs: prepare
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - name: Setup Node.js & Rust
        # ... setup steps
      - name: Build
        run: pnpm tauri build --target x86_64-unknown-linux-gnu
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: linux-artifacts
          path: |
            src-tauri/target/release/bundle/deb/*.deb
            src-tauri/target/release/bundle/appimage/*.AppImage

  build-windows:
    needs: prepare
    runs-on: windows-2022
    steps:
      - uses: actions/checkout@v4
      - name: Setup Node.js & Rust
        # ... setup steps
      - name: Build
        run: pnpm tauri build --target x86_64-pc-windows-msvc
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: windows-artifacts
          path: |
            src-tauri/target/release/bundle/nsis/*.exe
            src-tauri/target/release/bundle/msi/*.msi

  build-macos:
    needs: prepare
    runs-on: macos-14
    strategy:
      matrix:
        target: [x86_64-apple-darwin, aarch64-apple-darwin]
    steps:
      - uses: actions/checkout@v4
      - name: Setup Node.js & Rust
        # ... setup steps
      - name: Build
        run: pnpm tauri build --target ${{ matrix.target }}
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: macos-${{ matrix.target }}
          path: src-tauri/target/release/bundle/dmg/*.dmg

  create-release:
    needs: [prepare, build-linux, build-windows, build-macos]
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts
          merge-multiple: true
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: v${{ needs.prepare.outputs.version }}
          name: Release v${{ needs.prepare.outputs.version }}
          body: |
            ## Portal Desktop ${{ needs.prepare.outputs.version }}
            
            ### Downloads
            - **Linux**: `.deb` and `.AppImage`
            - **Windows**: `.exe` installer
            - **macOS**: `.dmg` disk image
          files: artifacts/**/*
          draft: false
          prerelease: ${{ needs.prepare.outputs.release_type == 'beta' }}
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Option 2: Using tauri-action (Simpler but Less Control)

The `tauri-action` can handle building and releasing automatically:

```yaml
name: Build and Release

on:
  push:
    branches: [main]

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        platform: [macos-latest, windows-latest, ubuntu-latest]
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '22'
      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Build Tauri App
        uses: tauri-apps/tauri-action@v0
        with:
          tagName: v__VERSION__
          releaseName: 'Release v__VERSION__'
          releaseBody: 'See the assets to download this version.'
          releaseDraft: true
          prerelease: false
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}
          TAURI_KEY_PASSWORD: ${{ secrets.TAURI_KEY_PASSWORD }}
```

**Note**: `tauri-action` is older and may not support Tauri v2 well. Manual approach is recommended.

## Platform-Specific Considerations

### Windows Builds

**Runner**: `windows-2022` or `windows-latest`

**Artifacts**:
- `src-tauri/target/release/bundle/nsis/*.exe` - NSIS installer
- `src-tauri/target/release/bundle/msi/*.msi` - Windows Installer

**Code Signing** (Optional but recommended):
```yaml
- name: Setup code signing certificate
  run: |
    echo "${{ secrets.WINDOWS_CERTIFICATE }}" | base64 -d > cert.pfx
  shell: bash
```

**Configuration in `tauri.conf.json`**:
```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_THUMBPRINT",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

### macOS Builds

**Runner**: `macos-14` or `macos-latest`

**Artifacts**:
- `src-tauri/target/release/bundle/dmg/*.dmg` - Disk image
- `src-tauri/target/release/bundle/macos/*.app` - Application bundle

**Universal Binaries** (Intel + Apple Silicon):
```yaml
strategy:
  matrix:
    target: [x86_64-apple-darwin, aarch64-apple-darwin]
```

Build both targets and create a universal binary, or release separate builds.

**Code Signing & Notarization** (Required for distribution):
```yaml
- name: Setup Apple certificates
  run: |
    echo "${{ secrets.MACOS_CERTIFICATE }}" | base64 -d > certificate.p12
    security create-keychain -p "${{ secrets.MACOS_KEYCHAIN_PASSWORD }}" build.keychain
    security default-keychain -s build.keychain
    security unlock-keychain -p "${{ secrets.MACOS_KEYCHAIN_PASSWORD }}" build.keychain
    security import certificate.p12 -k build.keychain -P "${{ secrets.MACOS_CERTIFICATE_PASSWORD }}" -T /usr/bin/codesign
    security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "${{ secrets.MACOS_KEYCHAIN_PASSWORD }}" build.keychain
```

**Configuration in `tauri.conf.json`**:
```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAMID)",
      "notarize": true,
      "hardenedRuntime": true,
      "gatekeeperAssess": false
    }
  }
}
```

**Required Secrets**:
- `APPLE_ID` - Apple Developer account email
- `APPLE_PASSWORD` - App-specific password
- `APPLE_TEAM_ID` - Developer Team ID
- `MACOS_CERTIFICATE` - Base64-encoded certificate
- `MACOS_CERTIFICATE_PASSWORD` - Certificate password

## Artifact Management

### Artifact Naming

Tauri generates artifacts with default names. You can rename them:

```yaml
- name: Rename artifacts
  run: |
    # Rename to include version
    find src-tauri/target/release/bundle -name "*.exe" -exec mv {} portal-desktop-${{ needs.prepare.outputs.version }}-windows.exe \;
```

Or use your existing `rename-artifacts.sh` script.

### Artifact Upload Strategy

**Option A: Upload per platform** (Recommended)
```yaml
- name: Upload artifacts
  uses: actions/upload-artifact@v4
  with:
    name: windows-artifacts
    path: src-tauri/target/release/bundle/**/*
```

**Option B: Upload entire bundle directory**
```yaml
- name: Upload artifacts
  uses: actions/upload-artifact@v4
  with:
    name: windows-artifacts
    path: src-tauri/target/release/bundle/
```

### Downloading All Artifacts for Release

```yaml
- name: Download all artifacts
  uses: actions/download-artifact@v4
  with:
    path: artifacts
    merge-multiple: true  # Merges all artifacts into one directory
```

## Release Creation

### Using softprops/action-gh-release

```yaml
- name: Create Release
  uses: softprops/action-gh-release@v1
  with:
    tag_name: v${{ needs.prepare.outputs.version }}
    name: Portal Desktop ${{ needs.prepare.outputs.version }}
    body: |
      ## What's New
      - Feature 1
      - Feature 2
    files: |
      artifacts/**/*.exe
      artifacts/**/*.dmg
      artifacts/**/*.deb
      artifacts/**/*.AppImage
    draft: false
    prerelease: ${{ needs.prepare.outputs.release_type == 'beta' }}
  env:
    GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

### Release Body Template

```yaml
body: |
  ## Portal Desktop ${{ needs.prepare.outputs.version }}
  
  **Release Type:** ${{ needs.prepare.outputs.release_type }}
  
  ### Downloads
  
  - **Linux**: [portal-desktop-${{ needs.prepare.outputs.version }}-linux.deb](link)
  - **Windows**: [portal-desktop-${{ needs.prepare.outputs.version }}-windows.exe](link)
  - **macOS**: [portal-desktop-${{ needs.prepare.outputs.version }}-macos.dmg](link)
  
  ### Installation
  
  See [Installation Guide](https://github.com/${{ github.repository }}/blob/main/docs/getting-started/INSTALLATION.md)
  
  ### Changelog
  
  See [Release Notes](https://github.com/${{ github.repository }}/blob/main/docs/status/RELEASE_NOTES.md)
```

## Best Practices

### 1. Version Management

- **Primary source**: `tauri.conf.json` version field
- **Sync**: Keep `package.json` and `Cargo.toml` in sync
- **Tagging**: Use semantic versioning (`v1.0.0`, `v1.0.0-beta.1`)

### 2. Conditional Releases

Only create releases on tags or main branch:

```yaml
create-release:
  if: startsWith(github.ref, 'refs/tags/v') || github.ref == 'refs/heads/main'
  needs: [build-linux, build-windows, build-macos]
```

### 3. Draft Releases

Create drafts first, then publish manually:

```yaml
draft: true  # Review before publishing
```

### 4. Prerelease Handling

```yaml
prerelease: ${{ contains(github.ref, 'beta') || contains(github.ref, 'alpha') }}
```

### 5. Artifact Retention

Set appropriate retention periods:

```yaml
retention-days: 90  # Keep artifacts for 90 days
```

### 6. Error Handling

Make artifact uploads non-blocking:

```yaml
- name: Upload artifacts
  uses: actions/upload-artifact@v4
  if: always()  # Upload even if build fails
  with:
    if-no-files-found: warn  # Don't fail if no artifacts
```

## Common Issues & Solutions

### Issue: Artifacts Not Found

**Solution**: Check artifact paths match Tauri output:
```bash
# List actual paths
find src-tauri/target/release/bundle -type f
```

### Issue: Release Creation Fails

**Solution**: Ensure `contents: write` permission:
```yaml
permissions:
  contents: write
```

### Issue: macOS Notarization Fails

**Solution**: 
- Verify Apple credentials are correct
- Check certificate is valid
- Ensure `APPLE_TEAM_ID` matches certificate

### Issue: Windows Code Signing Fails

**Solution**:
- Verify certificate is valid
- Check `certificateThumbprint` in `tauri.conf.json`
- Ensure certificate is installed in Windows runner

## Example: Complete Multi-Platform Workflow

See the reference implementation in your codebase:
- `logs-explorer/.github/workflows/release.yml` - Multi-platform example
- `waveterm/.github/workflows/build-helper.yml` - Artifact management example

## References

- [Tauri Action Documentation](https://github.com/tauri-apps/tauri-action)
- [GitHub Actions Artifacts](https://docs.github.com/en/actions/using-workflows/storing-workflow-data-as-artifacts)
- [Creating Releases](https://docs.github.com/en/rest/releases/releases)
- [softprops/action-gh-release](https://github.com/softprops/action-gh-release)

