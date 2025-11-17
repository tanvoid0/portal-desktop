/**
 * Learning domain exports
 */

export { learningService } from './services/learningService';
export { suggestionEngine } from './services/suggestionEngine';
export { patternCollector } from './services/patternCollector';
export { memoryService } from './services/memoryService';
export { learningStore, hasSuggestions, isLoading, hasError } from './stores/learningStore';
export { default as AutoActionBadge } from './components/AutoActionBadge.svelte';
export { default as SuggestionPanel } from './components/SuggestionPanel.svelte';
export type * from './types';
export type { MemoryStats, CleanupStats, CleanupPreview } from './services/memoryService';

