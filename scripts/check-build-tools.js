#!/usr/bin/env node
/**
 * Cross-platform build tools checker
 * Ensures required build tools are available for native dependencies
 */

import { execSync } from 'child_process';
import { platform } from 'os';

const PLATFORM = platform();

console.log(`🔍 Checking build tools for ${PLATFORM}...`);

function checkCommand(command, errorMessage) {
  try {
    execSync(command, { stdio: 'ignore' });
    console.log(`✅ ${command} found`);
    return true;
  } catch (error) {
    console.log(`❌ ${command} not found`);
    console.log(`   ${errorMessage}`);
    return false;
  }
}

function checkPython() {
  const pythonCommands = ['python3', 'python'];
  for (const cmd of pythonCommands) {
    if (checkCommand(`${cmd} --version`, 'Python is required for node-gyp')) {
      return true;
    }
  }
  return false;
}

function checkNodeVersion() {
  try {
    const nodeVersion = execSync('node --version', { encoding: 'utf8' }).trim();
    const expectedVersion = 'v22.12.0';
    
    console.log(`📋 Current Node.js version: ${nodeVersion}`);
    
    if (nodeVersion !== expectedVersion) {
      console.log(`⚠️  Warning: Expected Node.js ${expectedVersion}, but found ${nodeVersion}`);
      console.log('   This may cause build issues. Consider using nvm to switch versions:');
      console.log('   nvm install 22.12.0 && nvm use 22.12.0');
      return false;
    }
    
    console.log('✅ Node.js version is correct');
    return true;
  } catch (error) {
    console.log('❌ Could not check Node.js version');
    return false;
  }
}

function checkSpectreLibraries() {
  if (PLATFORM !== 'win32') {
    return true; // Not applicable on non-Windows platforms
  }
  
  try {
    // Check if Spectre-mitigated libraries are installed
    // This is a heuristic check - we'll look for the specific error pattern
    console.log('🔍 Checking for Spectre-mitigated libraries...');
    
    // Try to compile a simple test to see if Spectre libs are available
    const testCode = `
#include <windows.h>
int main() { return 0; }
`;
    
    const fs = require('fs');
    const path = require('path');
    const testFile = path.join(__dirname, '..', 'test_spectre.cpp');
    const exeFile = path.join(__dirname, '..', 'test_spectre.exe');
    
    try {
      fs.writeFileSync(testFile, testCode);
      
      // Try to compile with Spectre mitigation
      execSync(`cl.exe /Qspectre ${testFile} /Fe:${exeFile}`, { 
        stdio: 'pipe',
        shell: 'cmd.exe'
      });
      
      // Clean up
      if (fs.existsSync(testFile)) fs.unlinkSync(testFile);
      if (fs.existsSync(exeFile)) fs.unlinkSync(exeFile);
      
      console.log('✅ Spectre-mitigated libraries found');
      return true;
    } catch (error) {
      // Clean up on error
      if (fs.existsSync(testFile)) fs.unlinkSync(testFile);
      if (fs.existsSync(exeFile)) fs.unlinkSync(exeFile);
      
      console.log('❌ Spectre-mitigated libraries not found');
      console.log('   Install them from Visual Studio Installer:');
      console.log('   1. Open Visual Studio Installer');
      console.log('   2. Click "Modify" on VS 2022 Community');
      console.log('   3. Go to "Individual components" tab');
      console.log('   4. Search for "Spectre" and install the libraries');
      return false;
    }
  } catch (error) {
    console.log('⚠️  Could not check Spectre libraries (this is normal if VS is not in PATH)');
    return true; // Don't fail the check if we can't test
  }
}

function checkBuildTools() {
  let allGood = true;

  // Check Node.js version first
  if (!checkNodeVersion()) {
    allGood = false;
  }

  // Check Python (required on all platforms)
  if (!checkPython()) {
    allGood = false;
  }

  if (PLATFORM === 'win32') {
    // Windows: Check for Visual Studio Build Tools
    try {
      execSync('where msbuild', { stdio: 'ignore' });
      console.log('✅ MSBuild found');
    } catch (error) {
      console.log('❌ MSBuild not found');
      console.log('   Install Visual Studio Build Tools with C++ workload');
      console.log('   Or run: npm run install:windows');
      allGood = false;
    }
    
    // Check for Spectre-mitigated libraries (critical for node-pty)
    if (!checkSpectreLibraries()) {
      allGood = false;
    }
  } else if (PLATFORM === 'darwin') {
    // macOS: Check for Xcode Command Line Tools
    try {
      execSync('xcode-select -p', { stdio: 'ignore' });
      console.log('✅ Xcode Command Line Tools found');
    } catch (error) {
      console.log('❌ Xcode Command Line Tools not found');
      console.log('   Run: xcode-select --install');
      console.log('   Or run: npm run install:macos');
      allGood = false;
    }
  } else if (PLATFORM === 'linux') {
    // Linux: Check for gcc
    if (!checkCommand('gcc --version', 'Install build-essential package')) {
      allGood = false;
    }
  }

  return allGood;
}

// Main check
const buildToolsOk = checkBuildTools();

if (!buildToolsOk) {
  console.log('\n🚨 Build tools check failed!');
  console.log('\nTo fix this, run the appropriate installation script:');
  console.log('  Windows: npm run install:windows');
  console.log('  macOS:   npm run install:macos');
  console.log('  Linux:   npm run install:linux');
  console.log('\nOr follow the detailed guide in INSTALLATION.md');
  process.exit(1);
} else {
  console.log('\n🎉 All build tools are ready!');
  console.log('You can now run: npm run dev');
}
