# Portal Desktop - Cross-Platform Installation Guide

This guide ensures smooth installation of Portal Desktop across Windows, macOS, and Linux platforms.

## Prerequisites

Portal Desktop requires native compilation for some dependencies (`node-pty`, `ws`, `esbuild`). This means you need build tools installed on your system.

### Node.js Version Requirements
- **Required**: Node.js 22.12.0+ (see `.nvmrc`)
- **Why**: Modern dependencies require Node.js 20.19+ or 22.12+
- **Check**: `node --version` should return `v22.12.0` or higher
- **Fix**: Use nvm to switch versions: `nvm install 22.12.0 && nvm use 22.12.0`

## Quick Start

### Automated Installation (All Platforms)
```bash
# Single command that works on Windows, macOS, and Linux
npm run install
```

**PowerShell Users (Windows):**
```powershell
# If you get syntax errors, use PowerShell-specific commands
npm run install
# Or manually:
npm install
```

### Manual Installation
If the automated script doesn't work, follow the platform-specific steps below.

## Detailed Requirements

### Windows Requirements
- **Python 3.x** (from Microsoft Store or python.org)
- **Visual Studio Build Tools** or **Visual Studio Community**
  - Select "Desktop development with C++" workload
  - Includes MSVC compiler, Windows SDK, and CMake
- **Spectre-mitigated libraries** (CRITICAL for node-pty)
  - Open Visual Studio Installer
  - Click "Modify" on VS 2022 Community
  - Go to "Individual components" tab
  - Search for "Spectre"
  - Install "MSVC v143 - VS 2022 C++ x64/x86 Spectre-mitigated libs (Latest)"

### macOS Requirements
- **Xcode Command Line Tools**: `xcode-select --install`
- **Python 3.x** (usually pre-installed)
- **Node.js** (via Homebrew or official installer)

### Linux Requirements
- **Build Essential Tools** (gcc, g++, make)
- **Python 3.x**
- **Node.js** (via package manager or NodeSource)

#### Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install build-essential python3 python3-pip curl
```

#### CentOS/RHEL/Fedora:
```bash
# For newer versions (Fedora/RHEL 8+)
sudo dnf groupinstall "Development Tools"
sudo dnf install python3 python3-pip curl

# For older versions
sudo yum groupinstall "Development Tools"
sudo yum install python3 python3-pip curl
```

#### Arch Linux:
```bash
sudo pacman -S base-devel python curl
```

## Troubleshooting

### Common Issues

1. **"Python not found" error**
   - Ensure Python is in your PATH
   - On Windows, check "Add Python to PATH" during installation

2. **"MSBuild not found" (Windows)**
   - Install Visual Studio Build Tools
   - Or install Visual Studio Community with C++ workload

3. **"Spectre-mitigated libraries are required" (Windows)**
   - This is the most common issue with node-pty on Windows
   - Open Visual Studio Installer
   - Click "Modify" on VS 2022 Community
   - Go to "Individual components" tab
   - Search for "Spectre"
   - Install "MSVC v143 - VS 2022 C++ x64/x86 Spectre-mitigated libs (Latest)"
   - Restart your terminal and try `npm install` again

4. **"gcc not found" (Linux/macOS)**
   - Install Xcode Command Line Tools on macOS
   - Install build-essential on Linux

5. **Node.js version mismatch**
   - Expected: v22.12.0+, but found older version
   - Use nvm: `nvm install 22.12.0 && nvm use 22.12.0`
   - Or download Node.js 22.12.0+ from nodejs.org

6. **Permission errors**
   - Use `sudo` on Linux/macOS for system package installation
   - Run PowerShell as Administrator on Windows

### Alternative Solutions

If you continue having issues with native compilation:

1. **Use pre-built binaries** (if available)
2. **Use Docker** for consistent build environment
3. **Use GitHub Codespaces** or similar cloud development environments

## Development Commands

After successful installation:

```bash
# Start development server
npm run dev

# Build for production
npm run build

# Run Tauri development
npm run tauri:dev

# Build Tauri application
npm run tauri:build
```

## Dependencies Requiring Native Compilation

- **node-pty**: Pseudo-terminal functionality
- **ws**: WebSocket library
- **esbuild**: JavaScript bundler (has platform-specific binaries)

## Performance Tips

1. **Use pnpm** instead of npm for faster installs
2. **Cache node_modules** in CI/CD pipelines
3. **Use .nvmrc** to lock Node.js version
4. **Consider pre-built binaries** for production deployments

## Support

If you encounter issues:
1. Check the troubleshooting section above
2. Ensure all prerequisites are installed
3. Try clearing npm cache: `npm cache clean --force`
4. Delete node_modules and reinstall: `rm -rf node_modules && npm install`
