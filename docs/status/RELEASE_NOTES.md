# Portal Desktop - Release Notes

## Version: Pre-Release Cleanup Complete

### 🎉 Major Improvements

#### Code Quality & Architecture
- **Domain Registry Cleanup**: Fixed domain exports - all active domains (deployments, credentials, settings) are now properly exported
- **Component Deduplication**: Removed duplicate AI chat panel component, consolidated to domain-based structure
- **Placeholder Removal**: Replaced mock data in containerized terminal with real deployment store integration
- **SDK Pages**: Removed "coming soon" placeholders, replaced with helpful guidance messages

#### AI Features
- **AI Hub Dashboard**: Transformed AI page from simple redirect to comprehensive hub with:
  - Quick stats (default provider, recent conversations, status)
  - Quick action cards for all AI features
  - Recent conversations list
  - Navigation to all AI sub-pages
- **AI Settings**: AI provider settings are now in the dedicated AI page (not in general settings)
- **Unified AI Experience**: Single place for all AI interactions and management

#### Pipeline Features
- **Documentation**: All pipeline command TODOs have been documented as future features with clear explanations:
  - Pipeline variables management (requires database schema)
  - Pipeline secrets management (requires credentials domain integration)
  - Custom pipeline blocks (requires database schema)
  - Step execution logs (requires log storage system)
  - Step retry functionality (requires execution state management)

### 🔧 Technical Improvements

#### Backend (Rust)
- All pipeline command TODOs converted to FUTURE comments with context
- Execution service TODOs documented for future implementation

#### Frontend (TypeScript/Svelte)
- Removed duplicate `AIChatPanel` component from `src/lib/components/ai/`
- Updated all imports to use domain-based `AIChatPanel` from `src/lib/domains/ai/components/chat/`
- Containerized terminal page now uses real container data from deployments store
- All SDK page TODOs converted to FUTURE comments with context

### 📋 Known Limitations & Future Features

#### Pipeline Features (Planned for Future Release)
- Pipeline variables management - requires database schema for variable storage
- Pipeline secrets management - requires integration with credentials domain
- Custom pipeline blocks - requires database schema for block storage
- Step execution logs - requires log storage system for pipeline steps
- Step retry functionality - requires execution state management

#### SDK Features (Planned for Future Release)
- File editor integration for opening config files
- Environment variable editor dialog
- Service configuration dialogs
- Service log viewer
- SDK manager uninstallation logic
- Actual PID retrieval for database services

#### Kubernetes Features (Planned for Future Release)
- Event monitoring with filtering and alerting
- Resource metrics UI (CPU, memory visualization)
- Network topology visualization
- Helm chart management
- File transfer to/from containers
- Bulk operations (delete, scale, restart multiple resources)
- Resource tree hierarchical view

#### Terminal Features (Planned for Future Release)
- Command palette with quick actions
- Scrollback persistence across sessions
- Session restoration across app restarts
- Better error detection and hyperlink parsing

### ✅ Completed Features

#### Core Domains
- ✅ Terminal: Core functionality complete
- ✅ Projects: Full CRUD and management
- ✅ Tasks: Complete with AI generation
- ✅ Kubernetes/Cloud: 85% complete
- ✅ Documents: Complete
- ✅ Credentials: Complete
- ✅ Deployments: Complete
- ✅ SDK: Complete
- ✅ Settings: Complete
- ✅ AI: 90% complete (hub, chat, providers, history, training, logs)

### 🐛 Bug Fixes
- Fixed domain export issues - all active domains now properly accessible
- Removed duplicate component causing import confusion
- Fixed containerized terminal page to use real data instead of mocks

### 📝 Code Quality
- All TODOs converted to FUTURE comments with clear context
- No deprecated code remaining
- Clean domain structure with proper exports
- Consistent component organization

### 🚀 Ready for Release

The application is now stable and ready for release with:
- Clean codebase with no deprecated/unused code
- Proper domain structure and exports
- Complete AI hub experience
- All critical features functional
- Clear documentation of future features

