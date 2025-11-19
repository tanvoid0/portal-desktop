# Development Documentation

Architecture guides, patterns, and development best practices.

## 📚 Documentation

### [Complete Project Blueprint](./COMPLETE_PROJECT_BLUEPRINT.md)
Comprehensive project architecture and structure:
- Overall system design
- Domain-driven architecture
- Frontend and backend structure
- Component patterns
- Development workflow

### [Svelte Patterns Review](./SVELTE_PATTERNS_REVIEW.md)
Svelte 5 patterns and best practices:
- Component patterns
- State management
- Event handling
- TypeScript integration
- Performance optimization

### [Frontend Data Structure Fix](./FRONTEND_DATA_STRUCTURE_FIX.md)
Frontend data structure improvements:
- Data flow patterns
- Store architecture
- Type safety improvements

### [SDK Manager Improvements](./SDK_MANAGER_IMPROVEMENTS.md)
SDK manager enhancements:
- Architecture improvements
- Feature additions
- Performance optimizations

### [Theming Improvements](./THEMING_IMPROVEMENTS.md)
Theme system documentation:
- Theme architecture
- Customization guide
- Dark/light mode support
- Component theming

## 🏗️ Architecture Overview

Portal Desktop follows a **domain-driven architecture**:

```
src/
├── lib/
│   ├── components/     # Reusable UI components
│   ├── domains/       # Domain-specific modules
│   │   ├── ai/        # AI integration
│   │   ├── cloud/     # Cloud/K8s integration
│   │   ├── terminal/  # Terminal emulation
│   │   └── ...
│   └── utils/         # Shared utilities
└── routes/            # SvelteKit routes
```

## 🛠️ Development Workflow

1. **Start development:**
   ```bash
   npm run tauri:dev
   ```

2. **Type checking:**
   ```bash
   npm run check
   ```

3. **Linting:**
   ```bash
   npm run lint
   ```

4. **Testing:**
   ```bash
   npm run test
   ```

## 📝 Code Standards

- **TypeScript**: Strict mode enabled
- **Svelte**: Version 5 with runes
- **Styling**: Tailwind CSS
- **State**: Svelte stores + runes
- **Architecture**: Domain-driven design

## 🔗 Related Documentation

- [Getting Started](../getting-started/) - Setup and installation
- [Security](../security/) - Security best practices
- [Status](../status/) - Project status

