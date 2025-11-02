<!--
	Error Boundary Component for Production
	Handles component errors gracefully with fallback UI
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import type { Snippet } from 'svelte';
	import { Button } from './button';
	import { Alert, AlertDescription, AlertTitle } from './alert';
	import { TriangleAlert, RefreshCw } from 'lucide-svelte';
	import { cn } from '$lib/utils';

	interface Props {
		fallback?: Snippet;
		children?: Snippet;
		onError?: (error: Error, errorInfo: unknown) => void;
		showDetails?: boolean;
		class?: string;
	}

	let {
		fallback,
		children,
		onError,
		showDetails = false,
		class: className = ''
	}: Props = $props();

	let hasError = $state(false);
	let error = $state<Error | null>(null);
	let errorInfo = $state<unknown>(null);

	// Reset error state
	function resetError() {
		hasError = false;
		error = null;
		errorInfo = null;
	}

	// Handle component errors
	function handleError(event: Event) {
		const customEvent = event as CustomEvent<{ error: Error; errorInfo: unknown }>;
		hasError = true;
		error = customEvent.detail.error;
		errorInfo = customEvent.detail.errorInfo;
		
		// Log error for debugging
		console.error('Component Error:', error, errorInfo);
		
		// Call custom error handler
		if (error) {
			onError?.(error, errorInfo);
		}
	}

	// Retry function
	function retry() {
		resetError();
		// Force component re-render by updating a reactive variable
		// This is a simple way to trigger a re-render in Svelte
		window.location.reload();
	}

	onMount(() => {
		// Listen for component errors
		window.addEventListener('component-error', handleError as EventListener);
		
		return () => {
			window.removeEventListener('component-error', handleError as EventListener);
		};
	});
</script>

{#if hasError}
	<div class={cn("p-4", className)}>
		{#if fallback}
			{@render fallback()}
		{:else}
			<Alert variant="destructive">
				<TriangleAlert class="h-4 w-4" />
				<AlertTitle>Something went wrong</AlertTitle>
				<AlertDescription>
					An unexpected error occurred. Please try refreshing the page or contact support if the problem persists.
				</AlertDescription>
			</Alert>
			
			<div class="mt-4 flex gap-2">
				<Button variant="outline" size="sm" onclick={retry}>
					<RefreshCw class="h-4 w-4 mr-2" />
					Try Again
				</Button>
				<Button variant="outline" size="sm" onclick={() => window.location.reload()}>
					Refresh Page
				</Button>
			</div>

			{#if showDetails && error}
				<details class="mt-4">
					<summary class="cursor-pointer text-sm text-muted-foreground">
						Error Details
					</summary>
					<pre class="mt-2 text-xs bg-muted p-2 rounded overflow-auto">
						{error.message}
						{#if error.stack}
							\n\nStack Trace:\n
							{error.stack}
						{/if}
					</pre>
				</details>
			{/if}
		{/if}
	</div>
{:else}
	{@render children?.()}
{/if}

<style>
	/* Global error handling styles */
	:global(.error-boundary) {
		position: relative;
	}
</style>