# Windows Docker Containers Guide

## Windows Docker Images - They DO Exist!

Microsoft provides official Windows Docker base images:

### Available Windows Base Images

1. **Windows Server Core** (Recommended for builds)
   - `mcr.microsoft.com/windows/servercore:ltsc2025` (latest)
   - `mcr.microsoft.com/windows/servercore:ltsc2022`
   - Full Windows API support
   - ~5GB image size

2. **Nano Server** (Lightweight, limited)
   - `mcr.microsoft.com/windows/nanoserver:ltsc2025`
   - Minimal Windows API
   - ~300MB image size
   - May not work for Tauri builds (limited API)

3. **Windows** (Full desktop)
   - `mcr.microsoft.com/windows:ltsc2025`
   - Full Windows API + desktop features
   - Largest image size

## ⚠️ Critical Limitation

**Windows containers can ONLY run on Windows hosts!**

- ❌ Cannot run on Linux hosts
- ❌ Cannot run on macOS hosts
- ✅ Requires Windows Server or Windows 10/11 Pro
- ✅ Requires Docker Desktop configured for Windows containers

## Setting Up Windows Containers

### Prerequisites

1. **Windows Host Required**
   - Windows Server 2016+ or Windows 10/11 Pro
   - Docker Desktop for Windows installed

2. **Switch Docker to Windows Containers**
   ```powershell
   # In Docker Desktop, right-click icon → "Switch to Windows containers"
   # Or via command:
   & $Env:ProgramFiles\Docker\Docker\DockerCli.exe -SwitchDaemon
   ```

3. **Verify Windows Container Mode**
   ```powershell
   docker version
   # Should show "OS/Arch: windows/amd64"
   ```

### Building with Windows Containers

```powershell
# On Windows host with Windows containers enabled
docker-compose run --rm build-windows
```

## Why Use GitHub Actions Instead?

### Advantages of GitHub Actions

1. ✅ **Works from any host** - Linux, Mac, or Windows
2. ✅ **No local setup** - Runs in the cloud
3. ✅ **Native Windows environment** - Full Windows Server 2022
4. ✅ **Pre-configured** - All tools already installed
5. ✅ **Free for public repos** - No infrastructure costs

### When to Use Windows Docker Containers

- ✅ Local development/testing on Windows
- ✅ CI/CD on Windows Server infrastructure
- ✅ Offline builds
- ✅ Custom Windows build environments

## Comparison

| Method | Host Requirement | Setup Complexity | Best For |
|-------|-----------------|------------------|----------|
| **GitHub Actions** | Any (cloud) | ⭐ Easy | Production builds |
| **Windows Docker** | Windows only | ⭐⭐ Medium | Local Windows dev |
| **Linux Cross-compile** | Linux/Mac | ⭐⭐⭐ Hard | Limited use cases |

## Troubleshooting

### Error: "image operating system 'windows' cannot be used"

**Cause**: Docker is in Linux container mode on a non-Windows host.

**Solution**: 
- Use GitHub Actions instead, OR
- Switch to a Windows host and enable Windows containers

### Error: "The container operating system does not match the host"

**Cause**: Docker Desktop is in Linux container mode.

**Solution**:
```powershell
# Switch to Windows containers
& $Env:ProgramFiles\Docker\Docker\DockerCli.exe -SwitchDaemon
```

### Build Fails in Windows Container

**Possible causes**:
- Missing Visual Studio Build Tools
- Chocolatey installation issues
- Path issues with Windows vs Linux paths

**Solution**: Check the build logs and ensure all dependencies are installed correctly.

## Recommendation

**For most users**: Use GitHub Actions Windows runners (`windows-2022`)

**For Windows developers**: Windows Docker containers work great for local builds

**For Linux/Mac users**: Use GitHub Actions (Windows Docker won't work)

## Example: Using Windows Docker on Windows Host

```powershell
# 1. Ensure Windows containers are enabled
docker version  # Verify OS/Arch shows windows/amd64

# 2. Build Windows executable
docker-compose run --rm build-windows

# 3. Find artifacts
ls build-output/*.exe
```

## Summary

- ✅ Windows Docker images exist and work well
- ⚠️ But they require a Windows host
- ✅ GitHub Actions is recommended for most use cases
- ✅ Windows Docker is great for local Windows development

