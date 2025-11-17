<!--
	File picker component for selecting files or folders
	Integrates with Tauri's native file dialog
	Shared component that can be used across different domains
-->

<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { invoke } from '@tauri-apps/api/core';
	import { FileUp } from '@lucide/svelte';

	interface Props {
		value?: string;
		placeholder?: string;
		disabled?: boolean;
		required?: boolean;
		label?: string;
		description?: string;
		filters?: Array<{ name: string; extensions: string[] }>;
		selectFolder?: boolean; // If true, select folder instead of file
		onChange?: (path: string) => void;
	}

	let {
		value: valueProp = '',
		placeholder = '/path/to/file',
		disabled = false,
		required = false,
		label = 'File Path',
		description: descriptionProp,
		filters = [],
		selectFolder = false,
		onChange
	}: Props = $props();

	let value = $state(valueProp);

	const description = $derived(descriptionProp || (selectFolder ? 'Select a folder' : 'Select a file'));

	let isSelecting = $state(false);

	async function handleSelectFile() {
		if (disabled || isSelecting) return;

		try {
			isSelecting = true;
			const filterArray = filters.length > 0 
				? filters.map(f => [f.name, f.extensions] as [string, string[]])
				: undefined;
			
			const selectedPath = await invoke<string | null>('select_file', {
				title: label,
				filters: filterArray,
				defaultPath: value || undefined,
				selectFolder: selectFolder || undefined
			});
			
			if (selectedPath) {
				value = selectedPath;
				onChange?.(selectedPath);
			}
		} catch (error) {
			console.error('Failed to select file:', error);
		} finally {
			isSelecting = false;
		}
	}

	function handleInputChange(event: Event) {
		const target = event.target as HTMLInputElement;
		value = target.value;
		onChange?.(target.value);
	}

	// Update local value when prop changes
	$effect(() => {
		value = valueProp;
	});
</script>

<div class="space-y-2">
	{#if label}
		<Label for="file-input">{label}</Label>
	{/if}
	<div class="flex gap-2">
		<Input
			id="file-input"
			type="text"
			value={value}
			{placeholder}
			{disabled}
			{required}
			oninput={handleInputChange}
			class="flex-1"
		/>
		<Button
			type="button"
			variant="outline"
			onclick={handleSelectFile}
			disabled={disabled || isSelecting}
		>
			<FileUp class="h-4 w-4 mr-2" />
			{isSelecting ? 'Selecting...' : 'Browse'}
		</Button>
	</div>
	{#if description}
		<p class="text-sm text-muted-foreground">{description}</p>
	{/if}
</div>

