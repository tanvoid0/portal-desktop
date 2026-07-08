# Getting Started

Essential guides for setting up and using Portal Desktop.

## 📚 Guides

### [Agent Platform Setup](./AGENT_PLATFORM.md) **(required)**

Portal Desktop depends on [agent-platform](https://github.com/tanvoid0/agent-platform/) for all AI features. This guide covers:

- Architecture and what the desktop app delegates to the platform
- Cloning, running, and configuring agent-platform
- Connecting Portal Desktop (URL, workspace token, catalog)
- Troubleshooting common connection errors

### [Installation Guide](./INSTALLATION.md)

Complete cross-platform installation instructions for Windows, macOS, and Linux. Includes:

- Prerequisites and build tools setup
- Automated and manual installation methods
- Platform-specific requirements
- Troubleshooting common issues

### [Updates Setup](./UPDATES_SETUP.md)

Guide for configuring the auto-update system. Covers:

- Generating Tauri signing keys
- Configuring update endpoints
- Building and signing update artifacts
- Setting up GitHub Releases or custom hosting
- Automation with GitHub Actions

## 🚀 Quick Start

1. **Start agent-platform** (required for AI):

   ```bash
   git clone https://github.com/tanvoid0/agent-platform.git
   cd agent-platform && cp .env.example .env && pnpm install && pnpm dev:server
   ```

2. **Install Portal Desktop:**

   ```bash
   git clone https://github.com/tanvoid0/portal-desktop.git
   cd portal-desktop && pnpm install
   ```

3. **Start development:**

   ```bash
   pnpm tauri:dev
   ```

4. **Configure AI** in the app: **AI → Providers** → base URL `http://127.0.0.1:18410`

5. **Build for production:**
   ```bash
   pnpm tauri:build
   ```

## 📝 Next Steps

- Review [Security Documentation](../security/) before production deployment
- Check [Development Guides](../development/) for architecture details
- See [Status](../status/) for current project status
