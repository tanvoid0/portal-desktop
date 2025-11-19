/**
 * Terminal Domain Exports
 * 
 * Centralized exports for the terminal domain with full control
 */

// Types
export type * from './types';

// Services
export { TerminalService } from './services/terminalService';
export { terminalProcessManager, type ProcessConfig, type CommandInterceptor, type OutputParser } from './services/terminalProcessManager';

// Stores
export { terminalStore, terminalActions, activeTab, activeProcess, tabCount, runningProcesses } from './stores/terminalStore';
export { commandHistoryStore, type CommandHistoryEntry } from './stores/commandHistoryStore';

// Components
export { default as Terminal } from './components/Terminal.svelte';
export { default as CommandHistory } from './components/CommandHistory.svelte';
export { default as TabBar } from './components/TabBar.svelte';
export { default as TabContainer } from './components/TabContainer.svelte';
export { default as TerminalTabContainer } from './components/TerminalTabContainer.svelte';
export { default as ProjectTerminal } from './components/ProjectTerminal.svelte';
