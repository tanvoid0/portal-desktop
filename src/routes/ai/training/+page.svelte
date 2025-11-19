<script lang="ts">
	import { onMount } from 'svelte';
	import TrainingDataViewer from '$lib/domains/ai/components/training/TrainingDataViewer.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';
	import { invoke } from '@tauri-apps/api/core';
	import type { TrainingData } from '$lib/domains/ai/types/index.js';

	let trainingData = $state<TrainingData[]>([]);
	let isLoading = $state(false);

	onMount(async () => {
		await loadTrainingData();
	});

	async function loadTrainingData() {
		isLoading = true;
		try {
			trainingData = await invoke<TrainingData[]>('ai_list_training_data');
		} catch (error) {
			console.error('Failed to load training data:', error);
			toastActions.error('Failed to load training data', error);
		} finally {
			isLoading = false;
		}
	}

	async function handleDelete(data: TrainingData) {
		try {
			await invoke('ai_delete_training_data', { id: data.id });
			toastActions.success('Training data deleted');
			await loadTrainingData();
		} catch (error) {
			console.error('Failed to delete training data:', error);
			toastActions.error('Failed to delete training data', error);
		}
	}
</script>

<div class="h-full w-full p-6">
	<div class="space-y-4">
		<div>
			<h1 class="text-2xl font-bold">Training Data</h1>
			<p class="text-muted-foreground">View and manage AI training data</p>
		</div>
		{#if isLoading}
			<p class="text-muted-foreground">Loading...</p>
		{:else}
			<TrainingDataViewer data={trainingData} onDelete={handleDelete} />
		{/if}
	</div>
</div>

