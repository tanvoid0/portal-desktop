/**
 * Terminal Domain Exports
 * 
 * Centralized exports for the terminal domain with full control
 */

// Types
export * from './types';

// Services
export { TerminalService } from './services/terminalService';
export { terminalProcessManager, type ProcessConfig, type CommandInterceptor, type OutputParser } from './services/terminalProcessManager';

// Stores
export { terminalStore, terminalActions, activeTab, activeProcess, activeOutput } from './stores/terminalStore';
export { commandHistoryStore, type CommandHistoryEntry } from './stores/commandHistoryStore';

// Components
export { default as Terminal } from './components/Terminal.svelte';
export { default as CommandHistory } from './components/CommandHistory.svelte';
export { default as ContainerizedTerminal } from './components/ContainerizedTerminal.svelte';
