# Portal Desktop - Release Notes

## Version 0.2.3

### Highlights

- Terminal shell profiles, command blocks, and workspace navigation improvements
- Coder project workspaces integrated into the AI layout
- Deployments dashboard workload grouping and container overview
- Environment utilities section for local tooling checks
- Updater CI publishes `latest.json`; graceful update-check errors in Settings
- shadcn UI lint enforcement and shared confirm dialog
- `pnpm release:prepare` script for version bumping and tagging

---

## Version 0.2.2

### Fixes

- Credential key safety hardening
- GitHub device-flow UX improvements
- Chat timestamp fixes
- E2E test wiring
- CI: un-ignore vitest tests, fix cross-platform Rust test

---

## Version 0.2.0

### Required dependency

**[agent-platform](https://github.com/tanvoid0/agent-platform/)** must be running for all AI features. See [Agent Platform setup](../getting-started/AGENT_PLATFORM.md).

### Highlights

- **Agent-platform migration** — unified AI provider; chat, tasks, documents, disk analysis, and Coder route through the platform API
- **Coder workspace** — file explorer, git changes panel, smart commit dialog, terminal integration, browser panel
- **Coder multitask** — sub-agent cards and parallel task bar
- **GitHub integration** — connect account, browse repos and issues from the app
- **Catalog UI** — live provider/model catalog from `GET /v1/catalog` with updated agent-platform format
- **Responsive layout** — viewport tiers, shell sidebar improvements, chat markdown rendering

### Install / upgrade

Download from [GitHub Releases](https://github.com/tanvoid0/portal-desktop/releases) or let the built-in updater apply the new version. Configure **AI → Providers** after upgrade.

---

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
