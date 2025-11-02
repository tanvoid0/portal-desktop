<script lang="ts">
	import ModelCard from './ModelCard.svelte';
	import ProgressIndicator from './ProgressIndicator.svelte';
	import { Button } from './ui/button';
	
	export let models: any[] = [];
	export let isInstalled: boolean = false;
	export let loading: boolean = false;
	export let error: string | null = null;
	export let installingModel: string | null = null;
	export let installationProgress: number = 0;
	export let installationStatus: string = '';
	
	export let onInstall: ((modelName: string) => void) | undefined = undefined;
	export let onRemove: ((modelName: string) => void) | undefined = undefined;
	export let onView: ((modelName: string) => void) | undefined = undefined;
	export let onRetry: (() => void) | undefined = undefined;
	export let onBrowseAvailable: (() => void) | undefined = undefined;
</script>

{#if loading}
	<div class="flex items-center justify-center p-8">
		<div class="animate-spin rounded-full h-8 w-8 border-b-2 border-primary"></div>
		<span class="ml-2">Loading models...</span>
	</div>
{:else if error}
	<div class="text-center p-8">
		<p class="text-destructive">Error: {error}</p>
		{#if onRetry}
			<Button onclick={onRetry} class="mt-4">Retry</Button>
		{/if}
	</div>
{:else if models.length > 0 || installingModel}
	<div class="space-y-4">
		{#if installingModel}
			<ProgressIndicator 
				title="Installing {installingModel}"
				status={installationStatus}
				progress={installationProgress}
			/>
		{/if}
		
		<div class="space-y-2">
			{#each models as model}
				<ModelCard 
					{model}
					{isInstalled}
					{onInstall}
					{onRemove}
					{onView}
				/>
			{/each}
		</div>
		
		{#if models.length === 0 && !installingModel}
			<div class="text-center p-8">
				<p class="text-muted-foreground">
					{isInstalled ? 'No models installed yet.' : 'No models available.'}
				</p>
				{#if isInstalled && onBrowseAvailable}
					<Button onclick={onBrowseAvailable} class="mt-4">
						Browse Available Models
					</Button>
				{/if}
			</div>
		{/if}
	</div>
{:else}
	<div class="text-center p-8">
		<p class="text-muted-foreground">No models available.</p>
		{#if onRetry}
			<Button onclick={onRetry} class="mt-4">Load Models</Button>
		{/if}
	</div>
{/if}
