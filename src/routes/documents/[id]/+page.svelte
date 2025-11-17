<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import Icon from '@iconify/svelte';
	import { documentActions } from '$lib/domains/documents';
	import DocumentEditor from '$lib/domains/documents/components/DocumentEditor.svelte';
	import type { Document } from '$lib/domains/documents';
	import LoadingSpinner from '$lib/components/ui/loading-spinner.svelte';
	import { toastActions } from '$lib/domains/shared/stores/toastStore';

	let document = $state<Document | null>(null);
	let isLoading = $state(true);
	let error = $state<string | null>(null);

	const documentId = $derived(parseInt($page.params.id));

	onMount(async () => {
		await loadDocument();
	});

	async function loadDocument() {
		isLoading = true;
		error = null;
		try {
			const doc = await documentActions.getDocument(documentId);
			if (doc) {
				document = doc;
			} else {
				error = 'Document not found';
			}
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load document';
			toastActions.error('Failed to load document', error);
		} finally {
			isLoading = false;
		}
	}

	async function handleSave(savedDoc: Document) {
		document = savedDoc;
		toastActions.success('Document saved successfully');
	}

	async function handleDelete() {
		if (!document) return;
		if (confirm(`Are you sure you want to delete "${document.title}"?`)) {
			try {
				await documentActions.deleteDocument(document.id);
				toastActions.success('Document deleted successfully');
				goto('/documents');
			} catch (err) {
				toastActions.error(
					'Failed to delete document',
					err instanceof Error ? err.message : 'An unexpected error occurred'
				);
			}
		}
	}
</script>

<svelte:head>
	<title>{document ? document.title : 'Document'} - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto py-6">
	<div class="mb-4">
		<Button variant="ghost" onclick={() => goto('/documents')}>
			<Icon icon="lucide:arrow-left" class="h-4 w-4 mr-2" />
			Back to Documents
		</Button>
	</div>

	{#if isLoading}
		<div class="flex items-center justify-center py-12">
			<LoadingSpinner size="lg" text="Loading document..." />
		</div>
	{:else if error}
		<Card>
			<CardContent class="py-12">
				<div class="text-center space-y-4">
					<Icon icon="lucide:alert-circle" class="h-12 w-12 text-destructive mx-auto" />
					<div>
						<h3 class="text-lg font-semibold text-destructive">Failed to load document</h3>
						<p class="text-muted-foreground">{error}</p>
					</div>
					<Button onclick={loadDocument} variant="outline">
						<Icon icon="lucide:refresh" class="h-4 w-4 mr-2" />
						Try Again
					</Button>
				</div>
			</CardContent>
		</Card>
	{:else if document}
		<div class="space-y-4">
			<div class="flex items-center justify-between">
				<h1 class="text-2xl font-bold">{document.title}</h1>
				<Button variant="destructive" onclick={handleDelete}>
					<Icon icon="lucide:trash-2" class="h-4 w-4 mr-2" />
					Delete
				</Button>
			</div>
			<DocumentEditor document={document} onSave={handleSave} />
		</div>
	{/if}
</div>

