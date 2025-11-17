<!--
	Folder picker component for selecting directories
	Integrates with Tauri's native file dialog
-->

<script lang="ts">
	import { Button } from './button';
	import { Input } from './input';
	import { Label } from './label';
	import { invoke } from '@tauri-apps/api/core';
	import { FolderOpen } from '@lucide/svelte';

	interface Props {
		value?: string;
		placeholder?: string;
		disabled?: boolean;
		required?: boolean;
		label?: string;
		description?: string;
		onChange?: (path: string) => void;
	}

	let {
		value = $bindable(''),
		placeholder = '/path/to/your/project',
		disabled = false,
		required = false,
		label = 'Project Path',
		description = 'Select the directory where your project will be located',
		onChange
	}: Props = $props();

	let isSelecting = $state(false);

	async function handleSelectDirectory() {
		if (disabled || isSelecting) return;

		try {
			isSelecting = true;
			const selectedPath = await invoke<string | null>('select_directory');
			
			if (selectedPath) {
				value = selectedPath;
				onChange?.(selectedPath);
			}
		} catch (error) {
			console.error('Failed to select directory:', error);
		} finally {
			isSelecting = false;
		}
	}

	function handleInputChange(event: Event) {
		const target = event.target as HTMLInputElement;
		value = target.value;
		onChange?.(target.value);
	}
</script>

<div class="space-y-2">
	{#if label}
		<Label for="folder-picker-input" class="text-sm font-medium">
			{label}
			{#if required}
				<span class="text-red-500 ml-1">*</span>
			{/if}
		</Label>
	{/if}
	
	<div class="flex space-x-2">
		<Input
			id="folder-picker-input"
			bind:value={value}
			oninput={handleInputChange}
			{placeholder}
			{disabled}
			{required}
			class="flex-1"
		/>
		<Button
			type="button"
			variant="outline"
			size="sm"
			onclick={handleSelectDirectory}
			disabled={disabled || isSelecting}
			class="px-3"
			title="Browse for directory"
		>
			<FolderOpen class="h-4 w-4" />
			<span class="sr-only">Select Directory</span>
		</Button>
	</div>
	
	{#if description}
		<p class="text-xs text-muted-foreground">
			{description}
		</p>
	{/if}
</div>
