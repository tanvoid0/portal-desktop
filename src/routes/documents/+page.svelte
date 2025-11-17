<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import Icon from '@iconify/svelte';
	import { documentActions, documents, isLoading, error } from '$lib/domains/documents';
	import DocumentList from '$lib/domains/documents/components/DocumentList.svelte';
	import type { Document } from '$lib/domains/documents';
	import LoadingSpinner from '$lib/components/ui/loading-spinner.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';

	onMount(() => {
		documentActions.loadDocuments();
	});

	async function handleDocumentClick(doc: Document) {
		goto(`/documents/${doc.id}`);
	}

	async function handleDocumentDelete(doc: Document) {
		if (confirm(`Are you sure you want to delete "${doc.title}"?`)) {
			try {
				await documentActions.deleteDocument(doc.id);
				toastActions.success('Document deleted successfully');
			} catch (err) {
				toastActions.error(
					'Failed to delete document',
					err instanceof Error ? err.message : 'An unexpected error occurred'
				);
			}
		}
	}

	function handleCreateNew() {
		goto('/documents/create');
	}
</script>

<svelte:head>
	<title>Documents - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto py-6">
	<div class="space-y-6">
		<!-- Header -->
		<div class="flex items-center justify-between">
			<div>
				<h1 class="text-3xl font-bold tracking-tight flex items-center gap-2">
					<Icon icon="lucide:file-text" class="h-8 w-8" />
					Documents
				</h1>
				<p class="text-muted-foreground">
					Workspace documentation, notes, and knowledge base
				</p>
			</div>
			<div class="flex gap-2">
				<Button variant="outline" onclick={() => goto('/documents/generate')}>
					<Icon icon="lucide:sparkles" class="h-4 w-4 mr-2" />
					Generate with AI
				</Button>
				<Button onclick={handleCreateNew}>
					<Icon icon="lucide:plus" class="h-4 w-4 mr-2" />
					New Document
				</Button>
			</div>
		</div>

		<!-- Document List -->
		{#if $isLoading}
			<div class="flex items-center justify-center py-12">
				<LoadingSpinner size="lg" text="Loading documents..." />
			</div>
		{:else if $error}
			<Card>
				<CardContent class="py-12">
					<div class="text-center space-y-4">
						<Icon icon="lucide:alert-circle" class="h-12 w-12 text-destructive mx-auto" />
						<div>
							<h3 class="text-lg font-semibold text-destructive">Failed to load documents</h3>
							<p class="text-muted-foreground">{$error}</p>
						</div>
						<Button onclick={() => documentActions.loadDocuments()} variant="outline">
							<Icon icon="lucide:refresh" class="h-4 w-4 mr-2" />
							Try Again
						</Button>
					</div>
				</CardContent>
			</Card>
		{:else}
			<DocumentList
				documents={$documents.filter((d) => !d.isArchived)}
				onDocumentClick={handleDocumentClick}
				onDocumentDelete={handleDocumentDelete}
				onCreateNew={handleCreateNew}
			/>
		{/if}
	</div>
</div>
