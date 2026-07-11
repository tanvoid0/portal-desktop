@echo off
echo.
echo === GitHub Project Setup ===
echo.
echo Step 1: Approve gh CLI access for Projects
echo   Run this in your terminal if not already done:
echo   gh auth refresh -h github.com -s read:project,project
echo.
echo   Enter the device code shown, then press any key here to continue...
pause >nul
echo.
echo Step 2: Creating project and adding issues...
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0setup-github-project.ps1"
