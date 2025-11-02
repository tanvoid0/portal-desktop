# Multi-Tab Terminal System

This document describes the multi-tab terminal functionality implemented in the portal desktop application.

## Features

### Tab Management
- **Create New Tab**: Click the `+` button or press `Ctrl+T`
- **Close Tab**: Click the `×` button on a tab or press `Ctrl+W`
- **Switch Tabs**: Click on a tab or use keyboard shortcuts
- **Duplicate Tab**: Press `Ctrl+D` to duplicate the current tab

### Keyboard Shortcuts
- `Ctrl+T` - Create new tab
- `Ctrl+W` - Close current tab
- `Ctrl+D` - Duplicate current tab
- `Ctrl+Tab` - Switch to next tab
- `Ctrl+Shift+Tab` - Switch to previous tab
- `Ctrl+1-9` - Switch to tab by number (1-9)

### Tab Indicators
- **Status Colors**: Each tab has a colored border indicating its status:
  - 🟢 Green: Running
  - 🔴 Red: Error
  - ✅ Blue: Completed
  - 💀 Gray: Killed
  - 💻 Default: Idle

### Tab Information
- **Tooltips**: Hover over tabs to see detailed information including:
  - Tab name
  - Working directory
  - Status
  - Process ID (truncated)

## Architecture

### Components
- `MultiTabTerminal.svelte` - Main container managing multiple tabs
- `TerminalTabBar.svelte` - Tab bar with tab management controls
- `Terminal.svelte` - Individual terminal instance
- `ContainerizedTerminal.svelte` - Containerized view option

### State Management
- `terminalStore.ts` - Centralized state management for tabs and processes
- `terminalActions` - Actions for tab and process management
- Reactive stores for active tab, process, and output

### Data Flow
1. User creates/switches tabs via UI or keyboard shortcuts
2. `terminalActions` updates the `terminalStore`
3. Components reactively update based on store changes
4. Each tab maintains its own terminal process and output

## Usage

### Basic Usage
```svelte
<script>
  import { MultiTabTerminal } from '$lib/domains/terminal';
  
  let settings = {
    theme: 'dark',
    fontSize: 14,
    fontFamily: 'Monaco, Consolas, "Courier New", monospace',
    // ... other settings
  };
</script>

<MultiTabTerminal {settings} />
```

### With Containerized View
```svelte
<script>
  import { MultiTabTerminal } from '$lib/domains/terminal';
  
  let showContainerized = false;
  let settings = { /* ... */ };
</script>

<MultiTabTerminal {showContainerized} {settings} />
```

## Integration

The multi-tab terminal integrates with:
- **Terminal Service**: Backend communication via Tauri
- **Command History**: Per-tab command tracking
- **Profile System**: Shell detection and selection
- **Settings**: Global terminal configuration

## Future Enhancements

Potential improvements:
- Tab reordering (drag & drop)
- Tab groups/folders
- Tab persistence across sessions
- Custom tab names
- Tab-specific settings
- Split panes within tabs
