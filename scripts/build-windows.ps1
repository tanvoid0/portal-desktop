# Build script for Windows (.exe) - PowerShell version for Windows containers

$ErrorActionPreference = "Stop"

$VERSION = if ($env:VERSION) { $env:VERSION } else { 
    $tauriConfig = Get-Content -Path "src-tauri/tauri.conf.json" | ConvertFrom-Json
    $tauriConfig.version 
}
$RELEASE_TYPE = if ($env:RELEASE_TYPE) { $env:RELEASE_TYPE } else { "release" }

Write-Host "Building Windows artifacts..." -ForegroundColor Cyan
Write-Host "Version: $VERSION" -ForegroundColor Cyan
Write-Host "Release Type: $RELEASE_TYPE" -ForegroundColor Cyan

# Ensure we're in the project root
Set-Location C:\app

# Install dependencies
pnpm install --frozen-lockfile

# Build frontend
pnpm build

# Build Tauri app for Windows
# Note: This requires Windows host with Docker Desktop in Windows container mode
# For production, use GitHub Actions windows-2022 runners
pnpm tauri build --target x86_64-pc-windows-msvc

# Rename artifacts
& bash scripts/rename-artifacts.sh windows

Write-Host "Windows build complete!" -ForegroundColor Green

