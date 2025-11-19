#!/usr/bin/env node
/**
 * Process logo image for Tauri app icons and web assets
 * Generates all required sizes and formats
 */

import { execSync } from 'child_process';
import { existsSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

// Get the project root directory (parent of scripts directory)
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const rootDir = join(__dirname, '..');
const logoPath = join(rootDir, 'static', 'logo.png');
const iconsDir = join(rootDir, 'src-tauri', 'icons');
const staticDir = join(rootDir, 'static');

// Ensure directories exist
const { mkdirSync } = await import('fs');
try {
  mkdirSync(iconsDir, { recursive: true });
} catch (e) {
  // Directory might already exist
}

console.log('🚀 Processing logo for Portal Desktop...\n');

// Check if logo exists and is valid
if (!existsSync(logoPath)) {
  console.error(`❌ Logo not found at: ${logoPath}`);
  process.exit(1);
}

// Check if logo file has content
const { statSync } = await import('fs');
try {
  const stats = statSync(logoPath);
  if (stats.size === 0) {
    console.error(`❌ Logo file is empty at: ${logoPath}`);
    console.error('   Please ensure the logo.png file contains a valid image.');
    process.exit(1);
  }
} catch (error) {
  console.error(`❌ Cannot read logo file: ${error.message}`);
  process.exit(1);
}

console.log('📐 Creating square versions and resizing...\n');

// Function to create square icon with padding
function createSquareIcon(size, outputPath, padding = 0.1) {
  const paddingPixels = Math.floor(size * padding);
  const contentSize = size - (paddingPixels * 2);
  
  // Create square version with padding (centered)
  execSync(
    `convert "${logoPath}" ` +
    `-resize ${contentSize}x${contentSize} ` +
    `-background transparent ` +
    `-gravity center ` +
    `-extent ${size}x${size} ` +
    `"${outputPath}"`,
    { stdio: 'inherit' }
  );
}

// Function to create regular resize
function resizeIcon(size, outputPath) {
  execSync(
    `convert "${logoPath}" -resize ${size}x${size} "${outputPath}"`,
    { stdio: 'inherit' }
  );
}

try {
  // 1. Tauri app icons (square with padding)
  console.log('  ✓ Creating 32x32.png...');
  createSquareIcon(32, join(iconsDir, '32x32.png'));
  
  console.log('  ✓ Creating 128x128.png...');
  createSquareIcon(128, join(iconsDir, '128x128.png'));
  
  console.log('  ✓ Creating 128x128@2x.png (256x256)...');
  createSquareIcon(256, join(iconsDir, '128x128@2x.png'));
  
  // 2. Create icon.png (main icon, 512x512 for best quality)
  console.log('  ✓ Creating icon.png (512x512)...');
  createSquareIcon(512, join(iconsDir, 'icon.png'), 0.15);
  
  // 3. Create Windows .ico file (multi-resolution)
  console.log('  ✓ Creating icon.ico (Windows)...');
  execSync(
    `convert ` +
    `"${join(iconsDir, '32x32.png')}" ` +
    `"${join(iconsDir, '128x128.png')}" ` +
    `"${join(iconsDir, '128x128@2x.png')}" ` +
    `"${join(iconsDir, 'icon.png')}" ` +
    `"${join(iconsDir, 'icon.ico')}"`,
    { stdio: 'inherit' }
  );
  
  // 4. Create macOS .icns file (macOS only - requires iconutil)
  console.log('  ⚠ Creating icon.icns (macOS - optional)...');
  try {
    // Create temporary directory for icns creation
    const tempIcnsDir = join(iconsDir, 'icon.iconset');
    mkdirSync(tempIcnsDir, { recursive: true });
    
    // Generate all required sizes for icns
    const icnsSizes = [
      { size: 16, name: 'icon_16x16.png' },
      { size: 32, name: 'icon_16x16@2x.png' },
      { size: 32, name: 'icon_32x32.png' },
      { size: 64, name: 'icon_32x32@2x.png' },
      { size: 128, name: 'icon_128x128.png' },
      { size: 256, name: 'icon_128x128@2x.png' },
      { size: 256, name: 'icon_256x256.png' },
      { size: 512, name: 'icon_256x256@2x.png' },
      { size: 512, name: 'icon_512x512.png' },
      { size: 1024, name: 'icon_512x512@2x.png' }
    ];
    
    for (const { size, name } of icnsSizes) {
      createSquareIcon(size, join(tempIcnsDir, name));
    }
    
    // Convert iconset to icns (macOS only)
    execSync(
      `iconutil -c icns "${tempIcnsDir}" -o "${join(iconsDir, 'icon.icns')}"`,
      { stdio: 'inherit' }
    );
    
    // Clean up temporary directory
    execSync(`rm -rf "${tempIcnsDir}"`, { stdio: 'inherit' });
    console.log('    ✓ icon.icns created successfully');
  } catch (error) {
    console.log('    ⚠ icon.icns creation skipped (iconutil not available - run on macOS to generate)');
    // Clean up temp directory if it exists
    const tempIcnsDir = join(iconsDir, 'icon.iconset');
    if (existsSync(tempIcnsDir)) {
      execSync(`rm -rf "${tempIcnsDir}"`, { stdio: 'inherit' });
    }
  }
  
  // 5. Create favicon.png (32x32 for web)
  console.log('  ✓ Creating favicon.png...');
  createSquareIcon(32, join(staticDir, 'favicon.png'), 0.1);
  
  // 6. Optimize static/logo.png (resize to reasonable web size if too large)
  console.log('  ✓ Optimizing static/logo.png...');
  try {
    // Check file size first
    const { statSync } = await import('fs');
    const stats = statSync(logoPath);
    if (stats.size > 0) {
      // Create a temporary file for optimization
      const tempPath = join(staticDir, 'logo_temp.png');
      execSync(
        `convert "${logoPath}" -resize 512x512\> "${tempPath}"`,
        { stdio: 'inherit' }
      );
      // Replace original with optimized version
      execSync(`mv "${tempPath}" "${logoPath}"`, { stdio: 'inherit' });
    } else {
      console.log('    ⚠ static/logo.png is empty, skipping optimization');
    }
  } catch (error) {
    console.log('    ⚠ Could not optimize static/logo.png:', error.message);
  }
  
  // 7. Create additional square logo variants for Windows Store
  console.log('  ✓ Creating Windows Store logos...');
  const storeSizes = [
    { size: 30, name: 'Square30x30Logo.png' },
    { size: 44, name: 'Square44x44Logo.png' },
    { size: 71, name: 'Square71x71Logo.png' },
    { size: 89, name: 'Square89x89Logo.png' },
    { size: 107, name: 'Square107x107Logo.png' },
    { size: 142, name: 'Square142x142Logo.png' },
    { size: 150, name: 'Square150x150Logo.png' },
    { size: 284, name: 'Square284x284Logo.png' },
    { size: 310, name: 'Square310x310Logo.png' }
  ];
  
  for (const { size, name } of storeSizes) {
    createSquareIcon(size, join(iconsDir, name), 0.1);
  }
  
  // Create StoreLogo.png (50x50)
  createSquareIcon(50, join(iconsDir, 'StoreLogo.png'), 0.1);
  
  console.log('\n✅ Logo processing complete!');
  console.log('\n📦 Generated files:');
  console.log('  - Tauri app icons (32x32, 128x128, 128x128@2x)');
  console.log('  - icon.ico (Windows)');
  console.log('  - icon.icns (macOS)');
  console.log('  - favicon.png (web)');
  console.log('  - Windows Store logos');
  console.log('  - Optimized static/logo.png');
  
} catch (error) {
  console.error('\n❌ Error processing logo:', error.message);
  process.exit(1);
}

