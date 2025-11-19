# Portal Desktop

A cross-platform desktop application built with SvelteKit and Tauri, featuring AI integration, terminal emulation, and development tools.

## 🚀 Quick Start

### Prerequisites
This project requires native compilation for some dependencies. Please ensure you have the required build tools installed:

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

- **Cross-platform Desktop App** (Windows, macOS, Linux)
- **AI Integration** with Ollama support
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
- [Installation Guide](./docs/getting-started/INSTALLATION.md) - Detailed setup instructions
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
