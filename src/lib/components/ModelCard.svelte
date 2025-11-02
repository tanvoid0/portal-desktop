<script lang="ts">
	import { parseAndFormatSize } from '$lib/utils/fileSize';
	import { Badge } from './ui/badge';
	import { Button } from './ui/button';
	
	export let model: any;
	export let isInstalled: boolean = false;
	export let onInstall: ((modelName: string) => void) | undefined = undefined;
	export let onRemove: ((modelName: string) => void) | undefined = undefined;
	export let onView: ((modelName: string) => void) | undefined = undefined;
	
	function handleAction() {
		if (isInstalled && onRemove) {
			onRemove(model.name);
		} else if (!isInstalled && onInstall) {
			onInstall(model.name);
		}
	}
	
	function handleView() {
		if (onView) {
			onView(model.name);
		}
	}
</script>

<div class="flex items-center justify-between p-4 border rounded-lg hover:bg-gray-50 transition-colors">
	<div class="flex items-center gap-4">
		<div class="flex items-center gap-2">
			{#if isInstalled}
				<svg class="w-5 h-5 text-green-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"/>
				</svg>
			{:else}
				<svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1"/>
				</svg>
			{/if}
			<span class="font-medium">{model.name}</span>
		</div>
		<div class="text-sm text-muted-foreground">
			{#if model.size}
				{parseAndFormatSize(model.size)}
			{:else if model.parameter_size}
				{model.parameter_size}
			{:else}
				Unknown size
			{/if}
		</div>
		{#if model.family}
			<div class="text-xs text-muted-foreground bg-gray-100 px-2 py-1 rounded">
				{model.family}
			</div>
		{/if}
	</div>
	<div class="flex items-center gap-2">
		{#if isInstalled}
			<Badge variant="default" class="bg-green-100 text-green-800">
				<svg class="w-3 h-3 mr-1" fill="currentColor" viewBox="0 0 24 24">
					<path d="M5 13l4 4L19 7"/>
				</svg>
				Installed
			</Badge>
			{#if onView}
				<Button size="sm" variant="outline" onclick={handleView}>
					<svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"/>
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"/>
					</svg>
					View
				</Button>
			{/if}
			{#if onRemove}
				<Button size="sm" variant="destructive" onclick={handleAction}>
					<svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"/>
					</svg>
					Remove
				</Button>
			{/if}
		{:else}
			<Badge variant="outline" class="text-gray-600">
				Available in Ollama Library
			</Badge>
			{#if onInstall}
				<Button size="sm" onclick={handleAction}>
					<svg class="w-4 h-4 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"/>
					</svg>
					Install
				</Button>
			{/if}
		{/if}
	</div>
</div>
