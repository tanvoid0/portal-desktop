// Public API exports for k8s-navigation domain
// Note: Generic keyboard utilities are in src/lib/domains/shared

export * from './types';
export { useTableNavigation, type TableNavigationOptions } from './hooks/useTableNavigation';
export * from './hooks/useKeyboardShortcuts';
export * from './hooks/useCommandPalette';
export * from './hooks/useResourceActions';
export * from './hooks/useNamespaceShortcuts';
export * from './hooks/useK8sKeyboard'; // K8s-specific wrapper
export { default as CommandPalette } from './components/CommandPalette.svelte';
export { default as ShortcutsHelp } from './components/ShortcutsHelp.svelte';
export { default as KeyboardShortcutHint } from './components/KeyboardShortcutHint.svelte';
export * from './utils/shortcutParser';
export * from './utils/fuzzySearch';
export * from './utils/keyboardUtils';

// Re-export generic components from shared domain
export { default as KeyboardShortcutsPanel } from '$lib/domains/shared/components/KeyboardShortcutsPanel.svelte';
export { type KeyboardShortcut as GenericKeyboardShortcut, type KeyboardConfig, type KeyboardReturn } from '$lib/domains/shared/hooks/useKeyboard';
export * from './utils/keyboardConstants';

