# Build and Release Guide

This guide explains how to build Portal Desktop for all platforms and create releases.

## Overview

Portal Desktop uses Docker (for Linux) and GitHub Actions (for all platforms) for cross-platform builds. The build system automatically:
- Extracts version from `tauri.conf.json` or `package.json`
- Builds executables for Linux (.deb, .AppImage), Windows (.exe), and macOS (.dmg)
- Names artifacts with version and release type (beta/release)
- Publishes to GitHub Releases

## Platform Build Support

### ✅ Linux - Full Docker Support
- **Docker**: Fully supported
- **Base Image**: Ubuntu 24.04 (latest LTS)
- **Outputs**: `.deb` and `.AppImage`

### ⚠️ Windows - Docker Support (Requires Windows Host)
- **Docker**: ✅ Supported, but requires Windows host
- **Windows Images**: `mcr.microsoft.com/windows/servercore:ltsc2025` (latest)
- **Limitation**: Cannot run on Linux/Mac hosts
- **Recommended**: Use GitHub Actions `windows-2022` runners (works from any host)
- **Outputs**: `.exe` (NSIS installer)
- **See**: [Windows Docker Guide](./DOCKER_WINDOWS.md) for details

### ❌ macOS - No Docker Support
- **Docker**: Not supported (requires macOS host)
- **Required**: GitHub Actions `macos-14` runners
- **Outputs**: `.dmg` disk image

## Prerequisites

- Docker and Docker Compose (for local Linux builds)
- GitHub repository with Actions enabled
- Tauri signing keys (see [Security Documentation](../security/TAURI_KEY_MANAGEMENT.md))

## Local Builds

### Linux Build (Docker)

```bash
# Set version and release type
export VERSION="1.0.0"
export RELEASE_TYPE="release"  # or "beta"

# Build Linux artifacts
docker-compose run --rm build-linux
```

**Output:**
- `build-output/portal-desktop-1.0.0-release-linux.deb`
- `build-output/portal-desktop-1.0.0-release-linux.AppImage`

### Windows Build (Limited)

**Option 1: Windows Container (requires Windows host)**
```bash
# On Windows host with Docker Desktop
docker-compose run --rm build-windows
```

**Option 2: Cross-compilation from Linux (LIMITED)**
```bash
# May fail for some Tauri features
docker-compose run --rm build-windows-cross
```

**⚠️ Recommended**: Use GitHub Actions for Windows builds instead.

### macOS Build

**❌ Not supported in Docker** - macOS builds require:
- macOS host system
- Xcode installed
- Code signing certificates

**Use GitHub Actions** `macos-14` runners instead.

## GitHub Actions Workflow

### Automatic Release on Tag

1. **Create and push a version tag:**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **For beta releases:**
   ```bash
   git tag v1.0.0-beta.1
   git push origin v1.0.0-beta.1
   ```

3. **GitHub Actions will automatically:**
   - Build for all platforms using native runners:
     - Linux: `ubuntu-24.04`
     - Windows: `windows-2022`
     - macOS: `macos-14`
   - Extract version from tag
   - Determine release type (beta if tag contains "beta" or "alpha")
   - Create GitHub Release with all artifacts

### Manual Release

1. Go to **Actions** → **Build and Release** → **Run workflow**
2. Enter:
   - **Version**: `1.0.0` (without 'v' prefix)
   - **Release type**: `release` or `beta`
3. Click **Run workflow**

## Version Management

### Version Sources (Priority Order)

1. **Git tag** (if building from tag): `v1.0.0` → `1.0.0`
2. **tauri.conf.json**: `"version": "0.1.0"`
3. **package.json**: `"version": "0.0.1"`

### Syncing Versions

To keep versions in sync:

```bash
# Update version in tauri.conf.json (primary source)
# Then sync to package.json
VERSION=$(node -p "require('./src-tauri/tauri.conf.json').version")
npm version $VERSION --no-git-tag-version
```

## Release Types

### Release
- **Tag format**: `v1.0.0`
- **Artifact naming**: `portal-desktop-1.0.0-release-*.{ext}`
- **GitHub Release**: Published (not draft, not prerelease)

### Beta
- **Tag format**: `v1.0.0-beta.1` or `v1.0.0-alpha.1`
- **Artifact naming**: `portal-desktop-1.0.0-beta-*.{ext}`
- **GitHub Release**: Draft and prerelease

## Platform-Specific Notes

### Linux
- **Base**: Ubuntu 24.04 LTS
- Generates both `.deb` and `.AppImage`
- `.deb` for Debian/Ubuntu-based systems
- `.AppImage` for universal Linux distribution
- ✅ Fully supported in Docker

### Windows
- **Base**: Windows Server 2022
- Generates `.exe` installer (NSIS)
- May also generate `.msi` (Windows Installer)
- ⚠️ Docker builds are limited - use GitHub Actions Windows runners
- Requires Windows host for Docker containers

### macOS
- **Base**: macOS 14 (Sonoma)
- Generates `.dmg` disk image
- Supports both Intel (x86_64) and Apple Silicon (aarch64)
- ❌ Cannot build in Docker - requires macOS host
- ✅ Use GitHub Actions macOS runners

## GitHub Secrets

Required secrets for signing (see [Tauri Key Management](../security/TAURI_KEY_MANAGEMENT.md)):

- `TAURI_PRIVATE_KEY`: Private signing key
- `TAURI_KEY_PASSWORD`: Password (if key is password-protected)

## Troubleshooting

### Build Fails in Docker

- **Linux**: Ensure all system dependencies are installed
- **Windows**: Use GitHub Actions Windows runner instead of Docker
- **macOS**: Use GitHub Actions macOS runner (Docker not supported)

### Version Not Extracted

- Check `tauri.conf.json` has valid version
- Ensure `scripts/get-version.sh` is executable
- Verify Node.js can parse JSON files

### Artifacts Not Renamed

- Check `build-output/` directory exists
- Verify build completed successfully
- Check file permissions on scripts

### GitHub Release Not Created

- Verify `GITHUB_TOKEN` has write permissions
- Check workflow logs for errors
- Ensure tag doesn't already exist

## Best Practices

1. **Version Before Release**: Update version in `tauri.conf.json` before tagging
2. **Test Builds**: Test local Linux builds before pushing tags
3. **Use GitHub Actions**: For Windows and macOS (Docker has limitations)
4. **Signing Keys**: Keep private keys secure (use GitHub Secrets)
5. **Release Notes**: Update `docs/status/RELEASE_NOTES.md` before release
6. **Tag Format**: Use semantic versioning (`v1.0.0`, `v1.0.0-beta.1`)

## Example Workflow

```bash
# 1. Update version in tauri.conf.json
# 2. Commit changes
git add src-tauri/tauri.conf.json
git commit -m "Bump version to 1.0.0"

# 3. Create and push tag
git tag v1.0.0
git push origin v1.0.0

# 4. GitHub Actions automatically builds and releases
#    - Linux: ubuntu-24.04 runner
#    - Windows: windows-2022 runner
#    - macOS: macos-14 runner
# 5. Check Actions tab for progress
# 6. Verify release in Releases section
```

## Docker vs GitHub Actions

| Platform | Docker Support | GitHub Actions | Recommended |
|----------|---------------|----------------|-------------|
| Linux    | ✅ Full       | ✅ Full        | Either      |
| Windows  | ⚠️ Limited    | ✅ Full        | GitHub Actions |
| macOS    | ❌ None       | ✅ Full        | GitHub Actions |

**Recommendation**: Use Docker for local Linux builds, GitHub Actions for all production releases.
