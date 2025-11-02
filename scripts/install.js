#!/usr/bin/env node
/**
 * Cross-platform installation script for Portal Desktop
 * Automatically detects OS and installs required build tools
 */

import { execSync, spawn } from 'child_process';
import { platform, arch } from 'os';
import { createWriteStream } from 'fs';
import { pipeline } from 'stream/promises';
import { createReadStream } from 'fs';

const PLATFORM = platform();
const ARCH = arch();

console.log(`🚀 Portal Desktop - Cross-Platform Installation`);
console.log(`📱 Detected: ${PLATFORM} ${ARCH}\n`);

// Colors for console output
const colors = {
  reset: '\x1b[0m',
  bright: '\x1b[1m',
  red: '\x1b[31m',
  green: '\x1b[32m',
  yellow: '\x1b[33m',
  blue: '\x1b[34m',
  magenta: '\x1b[35m',
  cyan: '\x1b[36m'
};

function log(message, color = 'reset') {
  console.log(`${colors[color]}${message}${colors.reset}`);
}

function exec(command, options = {}) {
  try {
    // Use PowerShell-compatible execution on Windows
    if (PLATFORM === 'win32') {
      return execSync(command, { 
        stdio: 'pipe', 
        shell: 'powershell.exe',
        ...options 
      });
    }
    return execSync(command, { stdio: 'pipe', ...options });
  } catch (error) {
    return null;
  }
}

function checkCommand(command, description) {
  const result = exec(command);
  if (result) {
    log(`✅ ${description} found`, 'green');
    return true;
  } else {
    log(`❌ ${description} not found`, 'red');
    return false;
  }
}

async function installWindows() {
  log('🪟 Setting up Windows build environment...', 'cyan');
  
  let allGood = true;
  
  // Check Python
  if (!checkCommand('python --version', 'Python')) {
    log('📥 Please install Python:', 'yellow');
    log('   1. Microsoft Store: https://www.microsoft.com/store/apps/9P7QFQMJRFP7', 'blue');
    log('   2. Or download from: https://www.python.org/downloads/', 'blue');
    log('   Make sure to check "Add Python to PATH" during installation', 'yellow');
    allGood = false;
  }
  
  // Check Visual Studio Build Tools
  let msbuildFound = false;
  
  // Check if MSBuild is in PATH
  if (checkCommand('where msbuild', 'MSBuild')) {
    msbuildFound = true;
  } else {
    // Check common Visual Studio installation paths
    const vsPaths = [
      'C:\\Program Files\\Microsoft Visual Studio\\2022\\Community\\MSBuild\\Current\\Bin\\MSBuild.exe',
      'C:\\Program Files\\Microsoft Visual Studio\\2022\\Professional\\MSBuild\\Current\\Bin\\MSBuild.exe',
      'C:\\Program Files\\Microsoft Visual Studio\\2022\\Enterprise\\MSBuild\\Current\\Bin\\MSBuild.exe',
      'C:\\Program Files\\Microsoft Visual Studio\\2022\\BuildTools\\MSBuild\\Current\\Bin\\MSBuild.exe',
      'C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Community\\MSBuild\\Current\\Bin\\MSBuild.exe',
      'C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Professional\\MSBuild\\Current\\Bin\\MSBuild.exe',
      'C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\Enterprise\\MSBuild\\Current\\Bin\\MSBuild.exe',
      'C:\\Program Files (x86)\\Microsoft Visual Studio\\2019\\BuildTools\\MSBuild\\Current\\Bin\\MSBuild.exe'
    ];
    
    for (const path of vsPaths) {
      try {
        const fs = await import('fs');
        if (fs.existsSync(path)) {
          log(`✅ MSBuild found at: ${path}`, 'green');
          msbuildFound = true;
          break;
        }
      } catch (error) {
        // Continue checking other paths
      }
    }
  }
  
  if (!msbuildFound) {
    log('📥 Please install Visual Studio Build Tools:', 'yellow');
    log('   1. Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/', 'blue');
    log('   2. Select "Desktop development with C++" workload', 'yellow');
    log('   3. Or install Visual Studio Community with C++ workload', 'yellow');
    allGood = false;
  } else {
    // Check for Spectre-mitigated libraries (common issue with VS2022)
    log('⚠️  Note: If you encounter "Spectre-mitigated libraries" errors:', 'yellow');
    log('   1. Open Visual Studio Installer', 'blue');
    log('   2. Click "Modify" on your VS installation', 'blue');
    log('   3. Go to "Individual components" tab', 'blue');
    log('   4. Search for "Spectre" and install "MSVC v143 - VS 2022 C++ x64/x86 Spectre-mitigated libs"', 'blue');
  }
  
  if (allGood) {
    log('🎉 Windows build environment is ready!', 'green');
  }
  
  return allGood;
}

async function installMacOS() {
  log('🍎 Setting up macOS build environment...', 'cyan');
  
  let allGood = true;
  
  // Check Xcode Command Line Tools
  if (!checkCommand('xcode-select -p', 'Xcode Command Line Tools')) {
    log('📥 Installing Xcode Command Line Tools...', 'yellow');
    try {
      execSync('xcode-select --install', { stdio: 'inherit' });
      log('⏳ Please complete the Xcode installation dialog and run this script again', 'yellow');
      return false;
    } catch (error) {
      log('❌ Failed to install Xcode Command Line Tools', 'red');
      allGood = false;
    }
  }
  
  // Check Python
  if (!checkCommand('python3 --version', 'Python3')) {
    log('📥 Installing Python via Homebrew...', 'yellow');
    if (!checkCommand('brew --version', 'Homebrew')) {
      log('📥 Installing Homebrew first...', 'yellow');
      try {
        execSync('/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"', { stdio: 'inherit' });
      } catch (error) {
        log('❌ Failed to install Homebrew', 'red');
        allGood = false;
      }
    }
    
    if (allGood) {
      try {
        execSync('brew install python', { stdio: 'inherit' });
      } catch (error) {
        log('❌ Failed to install Python', 'red');
        allGood = false;
      }
    }
  }
  
  // Check Node.js
  if (!checkCommand('node --version', 'Node.js')) {
    log('📥 Installing Node.js via Homebrew...', 'yellow');
    try {
      execSync('brew install node', { stdio: 'inherit' });
    } catch (error) {
      log('❌ Failed to install Node.js', 'red');
      allGood = false;
    }
  }
  
  if (allGood) {
    log('🎉 macOS build environment is ready!', 'green');
  }
  
  return allGood;
}

async function installLinux() {
  log('🐧 Setting up Linux build environment...', 'cyan');
  
  let allGood = true;
  
  // Detect Linux distribution
  let distro = 'unknown';
  try {
    const osRelease = exec('cat /etc/os-release');
    if (osRelease) {
      const content = osRelease.toString();
      if (content.includes('Ubuntu') || content.includes('Debian')) {
        distro = 'debian';
      } else if (content.includes('CentOS') || content.includes('Red Hat') || content.includes('Fedora')) {
        distro = 'redhat';
      } else if (content.includes('Arch')) {
        distro = 'arch';
      }
    }
  } catch (error) {
    log('⚠️  Could not detect Linux distribution', 'yellow');
  }
  
  log(`📦 Detected distribution: ${distro}`, 'blue');
  
  // Install build tools based on distribution
  if (distro === 'debian') {
    log('📥 Installing build tools for Ubuntu/Debian...', 'yellow');
    try {
      execSync('sudo apt-get update', { stdio: 'inherit' });
      execSync('sudo apt-get install -y build-essential python3 python3-pip curl', { stdio: 'inherit' });
    } catch (error) {
      log('❌ Failed to install build tools', 'red');
      allGood = false;
    }
  } else if (distro === 'redhat') {
    log('📥 Installing build tools for CentOS/RHEL/Fedora...', 'yellow');
    try {
      if (checkCommand('dnf --version', 'DNF')) {
        execSync('sudo dnf groupinstall -y "Development Tools"', { stdio: 'inherit' });
        execSync('sudo dnf install -y python3 python3-pip curl', { stdio: 'inherit' });
      } else {
        execSync('sudo yum groupinstall -y "Development Tools"', { stdio: 'inherit' });
        execSync('sudo yum install -y python3 python3-pip curl', { stdio: 'inherit' });
      }
    } catch (error) {
      log('❌ Failed to install build tools', 'red');
      allGood = false;
    }
  } else if (distro === 'arch') {
    log('📥 Installing build tools for Arch Linux...', 'yellow');
    try {
      execSync('sudo pacman -S --noconfirm base-devel python curl', { stdio: 'inherit' });
    } catch (error) {
      log('❌ Failed to install build tools', 'red');
      allGood = false;
    }
  } else {
    log('⚠️  Unsupported Linux distribution. Please install manually:', 'yellow');
    log('   - build-essential or equivalent (gcc, g++, make)', 'blue');
    log('   - python3', 'blue');
    log('   - curl', 'blue');
    allGood = false;
  }
  
  // Check Node.js
  if (!checkCommand('node --version', 'Node.js')) {
    log('📥 Installing Node.js...', 'yellow');
    try {
      if (distro === 'debian') {
        execSync('curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -', { stdio: 'inherit' });
        execSync('sudo apt-get install -y nodejs', { stdio: 'inherit' });
      } else {
        log('❌ Please install Node.js manually for your distribution', 'red');
        allGood = false;
      }
    } catch (error) {
      log('❌ Failed to install Node.js', 'red');
      allGood = false;
    }
  }
  
  if (allGood) {
    log('🎉 Linux build environment is ready!', 'green');
  }
  
  return allGood;
}

function checkNodeVersion() {
  try {
    const nodeVersion = execSync('node --version', { encoding: 'utf8' }).trim();
    const expectedVersion = 'v22.12.0';
    
    log(`📋 Current Node.js version: ${nodeVersion}`, 'blue');
    
    if (nodeVersion !== expectedVersion) {
      log(`⚠️  Warning: Expected Node.js ${expectedVersion}, but found ${nodeVersion}`, 'yellow');
      log('   This may cause build issues. Consider using nvm to switch versions:', 'yellow');
      log('   nvm install 22.12.0 && nvm use 22.12.0', 'blue');
      return false;
    }
    
    log('✅ Node.js version is correct', 'green');
    return true;
  } catch (error) {
    log('❌ Could not check Node.js version', 'red');
    return false;
  }
}

async function installNodeDependencies() {
  log('📦 Installing Node.js dependencies...', 'cyan');
  
  // Check Node.js version first
  const versionOk = checkNodeVersion();
  if (!versionOk) {
    log('⚠️  Continuing with installation despite version mismatch...', 'yellow');
  }
  
  // Skip npm install if we're already in an install process
  if (process.env.npm_lifecycle_event === 'install' || process.env.npm_lifecycle_event === 'postinstall') {
    log('✅ Skipping npm install (already in install process)', 'green');
    return true;
  }
  
  try {
    // Use PowerShell-compatible execution on Windows
    if (PLATFORM === 'win32') {
      execSync('npm install', { 
        stdio: 'inherit',
        shell: 'powershell.exe'
      });
    } else {
      execSync('npm install', { stdio: 'inherit' });
    }
    log('✅ Node.js dependencies installed successfully!', 'green');
    return true;
  } catch (error) {
    log('❌ Failed to install Node.js dependencies', 'red');
    log('   Try running: npm cache clean --force', 'yellow');
    log('   If you see Spectre-mitigated libraries error:', 'yellow');
    log('   1. Open Visual Studio Installer', 'blue');
    log('   2. Click "Modify" on VS 2022 Community', 'blue');
    log('   3. Go to "Individual components" tab', 'blue');
    log('   4. Search for "Spectre" and install the libraries', 'blue');
    return false;
  }
}

async function main() {
  let buildToolsOk = false;
  
  switch (PLATFORM) {
    case 'win32':
      buildToolsOk = await installWindows();
      break;
    case 'darwin':
      buildToolsOk = await installMacOS();
      break;
    case 'linux':
      buildToolsOk = await installLinux();
      break;
    default:
      log(`❌ Unsupported platform: ${PLATFORM}`, 'red');
      process.exit(1);
  }
  
  if (!buildToolsOk) {
    log('\n🚨 Build tools installation failed!', 'red');
    log('Please follow the manual installation steps above.', 'yellow');
    process.exit(1);
  }
  
  // Install Node.js dependencies
  const depsOk = await installNodeDependencies();
  
  if (depsOk) {
    log('\n🎉 Portal Desktop installation complete!', 'green');
    log('🚀 You can now run: npm run dev', 'cyan');
  } else {
    log('\n❌ Installation incomplete. Please check the errors above.', 'red');
    process.exit(1);
  }
}

// Run the installation
main().catch(error => {
  log(`❌ Installation failed: ${error.message}`, 'red');
  process.exit(1);
});
