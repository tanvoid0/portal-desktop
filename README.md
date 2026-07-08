# Portal Desktop

A cross-platform desktop application built with SvelteKit and Tauri — terminal emulation, development tools, Coder agent workspace, and AI features powered by [agent-platform](https://github.com/tanvoid0/agent-platform/).

**Downloads:** [GitHub Releases](https://github.com/tanvoid0/portal-desktop/releases) (Windows, macOS, Linux; auto-update built in)

## Required: Agent Platform

All AI features (chat, tasks, documents, Coder, disk analysis) require a running [**agent-platform**](https://github.com/tanvoid0/agent-platform/) instance. Portal Desktop is the client; LLM providers are configured on the platform.

```bash
# Terminal 1 — AI backend (required)
git clone https://github.com/tanvoid0/agent-platform.git
cd agent-platform && cp .env.example .env && pnpm install && pnpm dev:server

# Terminal 2 — Portal Desktop
git clone https://github.com/tanvoid0/portal-desktop.git
cd portal-desktop && pnpm install && pnpm tauri:dev
```

Then open **AI → Providers** in the app and point at `http://127.0.0.1:18410`. Full setup: [Agent Platform guide](./docs/getting-started/AGENT_PLATFORM.md).

## 🚀 Quick Start

### Prerequisites

This project requires native compilation for some dependencies. Please ensure you have the required build tools installed:

- **[agent-platform](https://github.com/tanvoid0/agent-platform/)** — running locally or via Docker (see above)
- **Windows**: Python 3.x + Visual Studio Build Tools
- **macOS**: Xcode Command Line Tools + Python 3.x
- **Linux**: build-essential + Python 3.x

### Installation

#### Automated Setup (Recommended)

```bash
# Single command for all platforms
npm run install
```

#### Manual Setup

```bash
# Install dependencies (will check build tools automatically)
npm install
```

### Development

```bash
# Start development server
npm run dev

# Start Tauri development (desktop app)
npm run tauri:dev
```

### Building

```bash
# Build web version
npm run build

# Build desktop application
npm run tauri:build
```

## 📋 Features

- **Cross-platform Desktop App** (Windows, macOS, Linux) with [GitHub Releases](https://github.com/tanvoid0/portal-desktop/releases) auto-update
- **AI via [agent-platform](https://github.com/tanvoid0/agent-platform/)** — chat, tasks, documents, catalog-driven model selection
- **Coder workspace** — agent turns, git changes, multitask sub-agents, terminal integration
- **GitHub integration** — connect account, browse repos and issues
- **Terminal Emulation** with xterm.js
- **Development Tools** and SDK management
- **Modern UI** with Tailwind CSS and Svelte 5

## 🛠️ Dependencies

This project uses native dependencies that require compilation:

- `node-pty` - Pseudo-terminal functionality
- `ws` - WebSocket library
- `esbuild` - JavaScript bundler

## 📖 Documentation

Comprehensive documentation is available in the [`docs/`](./docs/) directory:

- **[Getting Started](./docs/getting-started/)** - Installation and setup guides
- **[Security](./docs/security/)** - Security audits, fixes, and best practices
- **[Development](./docs/development/)** - Architecture and development guides
- **[Status](./docs/status/)** - Project status and release notes

**Quick Links:**

- [Agent Platform setup](./docs/getting-started/AGENT_PLATFORM.md) - **Required** AI backend ([github.com/tanvoid0/agent-platform](https://github.com/tanvoid0/agent-platform/))
- [Installation Guide](./docs/getting-started/INSTALLATION.md) - Detailed setup instructions
- [GitHub Releases](./docs/getting-started/GITHUB_RELEASES_TAURI.md) - Build and publish desktop releases
- [Security Documentation](./docs/security/) - Security reviews and fixes
- [Project Blueprint](./docs/development/COMPLETE_PROJECT_BLUEPRINT.md) - Architecture overview

## 🔧 Troubleshooting

If you encounter build issues:

1. **Check build tools**: The postinstall script will verify your environment
2. **Follow platform-specific guides** in `INSTALLATION.md`
3. **Clear cache**: `npm cache clean --force`
4. **Fresh install**: Delete `node_modules` and reinstall

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test across platforms
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License.
