# Auto-Update Setup Guide

This guide explains how to set up the auto-update feature for Portal Desktop.

## Overview

The application uses Tauri's built-in updater plugin to check for and install updates. Updates can be hosted on GitHub Releases or a custom server (e.g., Vercel).

## Prerequisites

- Tauri CLI installed (`npm install -g @tauri-apps/cli`)
- A GitHub repository (for GitHub Releases) or hosting service (for custom hosting)
- Access to build the application for all target platforms

## Step 1: Generate Signing Keys

Tauri requires signing keys to ensure update integrity. Generate a key pair using the Tauri CLI:

```bash
npm run tauri signer generate -- -w ~/.tauri/portal-desktop.key
```

This command will:
- Create a private key at `~/.tauri/portal-desktop.key` (keep this secure and private!)
- Display the public key in the terminal (you'll need this for the configuration)

**Important:** 
- Never commit the private key to version control
- Store the private key securely (consider using a password manager or secure vault)
- The public key is safe to share and will be included in the configuration

## Step 2: Configure the Updater

In Tauri v2, the updater plugin is configured programmatically. The endpoints and public key can be set via environment variables:

### Option 1: Environment Variables (Recommended)

Set the following environment variables before building or running:

```bash
export TAURI_UPDATER_ENDPOINTS='["https://github.com/YOUR_USERNAME/YOUR_REPOSITORY/releases/latest/download/latest.json"]'
export TAURI_UPDATER_PUBKEY="YOUR_PUBLIC_KEY_HERE"
```

Replace:
- `YOUR_USERNAME` with your GitHub username
- `YOUR_REPOSITORY` with your repository name
- `YOUR_PUBLIC_KEY_HERE` with the public key generated in Step 1

### Option 2: Configure in Code (Alternative)

You can also configure the updater by modifying `src-tauri/src/lib.rs` to pass endpoints and pubkey to the builder (if the API supports it).

### For Vercel Hosting (Future)

If you want to host updates on Vercel instead of GitHub Releases:

1. Create a `latest.json` file in your Vercel project's `public` directory
2. Update the endpoint environment variable:
   ```bash
   export TAURI_UPDATER_ENDPOINTS='["https://your-app.vercel.app/latest.json"]'
   ```

## Step 3: Build and Sign Update Artifacts

Before building, set the following environment variables:

```bash
export TAURI_PRIVATE_KEY="$(cat ~/.tauri/portal-desktop.key)"
export TAURI_KEY_PASSWORD=""  # Optional: if your key is password-protected
```

Then build the application:

```bash
npm run tauri:build
```

This will generate:
- Platform-specific installers (`.msi` for Windows, `.AppImage` for Linux, `.dmg` for macOS)
- Update bundles (`.zip` files)
- Signature files (`.sig` files)

The update artifacts will be in:
- `src-tauri/target/release/bundle/` (for the installer)
- `src-tauri/target/release/` (for update bundles and signatures)

## Step 4: Create Update Manifest

For each release, create a `latest.json` file with the following structure:

```json
{
  "version": "0.1.1",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2025-01-XXT00:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "CONTENT_OF_WINDOWS_SIG_FILE",
      "url": "https://github.com/YOUR_USERNAME/YOUR_REPOSITORY/releases/download/v0.1.1/portal-desktop_0.1.1_x64-setup.msi.zip"
    },
    "linux-x86_64": {
      "signature": "CONTENT_OF_LINUX_SIG_FILE",
      "url": "https://github.com/YOUR_USERNAME/YOUR_REPOSITORY/releases/download/v0.1.1/portal-desktop_0.1.1_amd64.AppImage.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "CONTENT_OF_MACOS_SIG_FILE",
      "url": "https://github.com/YOUR_USERNAME/YOUR_REPOSITORY/releases/download/v0.1.1/portal-desktop.app.tar.gz"
    },
    "darwin-aarch64": {
      "signature": "CONTENT_OF_MACOS_ARM_SIG_FILE",
      "url": "https://github.com/YOUR_USERNAME/YOUR_REPOSITORY/releases/download/v0.1.1/portal-desktop_aarch64.app.tar.gz"
    }
  }
}
```

### Getting Signatures

The signature content is the text content of the `.sig` files generated during build. For example:

```bash
cat src-tauri/target/release/portal-desktop_0.1.1_x64-setup.msi.zip.sig
```

Copy the entire output and paste it as the `signature` value in `latest.json`.

## Step 5: Host Updates

### GitHub Releases

1. Create a new release on GitHub
2. Upload all update bundles (`.zip` files) and their corresponding `.sig` files
3. Upload the `latest.json` file
4. Tag the release with the version (e.g., `v0.1.1`)

The `latest.json` file should be accessible at:
```
https://github.com/YOUR_USERNAME/YOUR_REPOSITORY/releases/latest/download/latest.json
```

### Vercel (Alternative)

1. Upload `latest.json` to your Vercel project's `public` directory
2. Upload update bundles to a CDN or storage service (e.g., AWS S3, Cloudflare R2)
3. Update the URLs in `latest.json` to point to your CDN/storage URLs
4. Deploy to Vercel

## Step 6: Test Updates

1. Build and run the application with an older version
2. Navigate to Settings > Updates
3. Click "Check for Updates"
4. Verify that the update is detected
5. Click "Install Update"
6. Verify that the application restarts with the new version

## Automation (GitHub Actions)

You can automate the release process using GitHub Actions. Here's a basic example:

```yaml
name: Release

on:
  push:
    tags:
      - v*

jobs:
  build:
    strategy:
      matrix:
        platform: [macos-latest, ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: 20
      - name: Install dependencies
        run: npm install
      - name: Build application
        env:
          TAURI_PRIVATE_KEY: ${{ secrets.TAURI_PRIVATE_KEY }}
        run: npm run tauri:build
      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: release-${{ matrix.platform }}
          path: src-tauri/target/release/bundle/
```

## Troubleshooting

### Updates Not Detected

- Verify the `latest.json` file is accessible at the configured endpoint
- Check that the version in `latest.json` is higher than the current app version
- Ensure the public key in `tauri.conf.json` matches the one used to sign updates
- Verify the endpoint URL is correct

### Installation Fails

- Check that signature files match the update bundles
- Verify the private key used for signing matches the public key in config
- Ensure update bundles are accessible at the URLs specified in `latest.json`
- Check application logs for detailed error messages

### Signature Verification Fails

- Ensure the public key in `tauri.conf.json` is correct
- Verify the signature content in `latest.json` matches the `.sig` file exactly
- Check that the update bundle hasn't been modified after signing

## Security Considerations

1. **Private Key Security**: Never commit the private key to version control. Use GitHub Secrets or similar services for CI/CD.
2. **HTTPS Only**: Always use HTTPS for update endpoints to prevent man-in-the-middle attacks.
3. **Signature Verification**: The updater automatically verifies signatures. Never disable this feature.
4. **Version Validation**: The updater only installs versions newer than the current version.

## Future Enhancements

- Automatic update checks on application startup (currently manual only)
- Background update downloads
- Update notifications
- Rollback functionality
- Delta updates for smaller downloads

