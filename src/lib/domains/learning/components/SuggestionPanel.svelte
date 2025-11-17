<script lang="ts">
	import { onMount } from 'svelte';
	import { suggestionEngine, learningStore } from '@/lib/domains/learning';
	import type { PatternType, Suggestion } from '@/lib/domains/learning/types';
	import { logger } from '@/lib/domains/shared/services/logger';
	import { Button } from '@/lib/components/ui/button';

	interface Props {
		patternType: PatternType;
		context?: string;
		maxSuggestions?: number;
		onSuggestionAccepted?: (suggestion: Suggestion) => void;
		onSuggestionRejected?: (suggestion: Suggestion) => void;
	}

	const {
		patternType,
		context,
		maxSuggestions = 5,
		onSuggestionAccepted,
		onSuggestionRejected,
	}: Props = $props();

	const log = logger.createScoped('SuggestionPanel');

	let suggestions: Suggestion[] = $state([]);
	let loading = $state(false);
	let error: string | null = $state(null);

	const loadSuggestions = async () => {
		loading = true;
		error = null;

		try {
			const fetched = await suggestionEngine.getContextualSuggestions(patternType, context);
			suggestions = fetched.slice(0, maxSuggestions);
			learningStore.setSuggestions(suggestions);
			log.info('Suggestions loaded', { count: suggestions.length });
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load suggestions';
			log.error('Failed to load suggestions', err);
		} finally {
			loading = false;
		}
	};

	const handleAccept = async (suggestion: Suggestion) => {
		try {
			await suggestionEngine.recordSuggestionAccepted(suggestion.pattern_id);
			onSuggestionAccepted?.(suggestion);
			log.info('Suggestion accepted', { pattern_id: suggestion.pattern_id });
		} catch (err) {
			log.error('Failed to record acceptance', err);
		}
	};

	const handleReject = async (suggestion: Suggestion) => {
		try {
			await suggestionEngine.recordSuggestionRejected(suggestion.pattern_id);
			onSuggestionRejected?.(suggestion);
			// Remove from local list
			suggestions = suggestions.filter((s) => s.pattern_id !== suggestion.pattern_id);
			log.info('Suggestion rejected', { pattern_id: suggestion.pattern_id });
		} catch (err) {
			log.error('Failed to record rejection', err);
		}
	};

	onMount(() => {
		loadSuggestions();
	});
</script>

<div class="suggestion-panel">
	{#if loading}
		<div class="loading">Loading suggestions...</div>
	{:else if error}
		<div class="error">{error}</div>
	{:else if suggestions.length === 0}
		<div class="empty">No suggestions available</div>
	{:else}
		<div class="suggestions-list">
			{#each suggestions as suggestion (suggestion.pattern_id)}
				<div class="suggestion-item">
					<div class="suggestion-content">
						<div class="suggestion-header">
							<span class="suggestion-title">
								{JSON.stringify(suggestion.pattern_data)}
							</span>
							<span class="suggestion-meta">
								Success rate: {(suggestion.success_rate * 100).toFixed(0)}% | Used: {suggestion.frequency}x
							</span>
						</div>
						{#if suggestion.context}
							<div class="suggestion-context">Context: {suggestion.context}</div>
						{/if}
					</div>
					<div class="suggestion-actions">
						<Button
							variant="default"
							size="sm"
							onclick={() => handleAccept(suggestion)}
							class="btn-accept"
						>
							Accept
						</Button>
						<Button
							variant="outline"
							size="sm"
							onclick={() => handleReject(suggestion)}
							class="btn-reject"
						>
							Reject
						</Button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.suggestion-panel {
		padding: 1rem;
		border: 1px solid var(--border);
		border-radius: 0.5rem;
		background: var(--background);
	}

	.loading,
	.error,
	.empty {
		padding: 1rem;
		text-align: center;
		color: var(--muted-foreground);
	}

	.error {
		color: var(--destructive);
	}

	.suggestions-list {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.suggestion-item {
		padding: 0.75rem;
		border: 1px solid var(--border);
		border-radius: 0.375rem;
		background: var(--card);
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: 1rem;
	}

	.suggestion-content {
		flex: 1;
	}

	.suggestion-header {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
	}

	.suggestion-title {
		font-weight: 500;
		color: var(--foreground);
	}

	.suggestion-meta {
		font-size: 0.875rem;
		color: var(--muted-foreground);
	}

	.suggestion-context {
		margin-top: 0.5rem;
		font-size: 0.875rem;
		color: var(--muted-foreground);
		font-style: italic;
	}

	.suggestion-actions {
		display: flex;
		gap: 0.5rem;
	}
</style>

