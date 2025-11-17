<script lang="ts">
	import { Input } from '$lib/components/ui/input';
	import { Button } from '$lib/components/ui/button';
	import Icon from '@iconify/svelte';
	import DocumentCard from './DocumentCard.svelte';
	import type { Document } from '../types';

	interface Props {
		documents: Document[];
		onDocumentClick?: (doc: Document) => void;
		onDocumentDelete?: (doc: Document) => void;
		onCreateNew?: () => void;
	}

	let { documents, onDocumentClick, onDocumentDelete, onCreateNew }: Props = $props();

	let searchQuery = $state('');
	const filteredDocuments = $derived(
		searchQuery.trim()
			? documents.filter(
					(doc) =>
						doc.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
						doc.content.toLowerCase().includes(searchQuery.toLowerCase())
				)
			: documents
	);
</script>

<div class="space-y-4">
	<div class="flex items-center gap-2">
		<div class="relative flex-1">
			<Icon
				icon="lucide:search"
				class="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground"
			/>
			<Input
				type="text"
				placeholder="Search documents..."
				bind:value={searchQuery}
				class="pl-10"
			/>
		</div>
		{#if onCreateNew}
			<Button onclick={onCreateNew}>
				<Icon icon="lucide:plus" class="h-4 w-4 mr-2" />
				New Document
			</Button>
		{/if}
	</div>

	{#if filteredDocuments.length === 0}
		<div class="text-center py-12 text-muted-foreground">
			<Icon icon="lucide:file-text" class="h-12 w-12 mx-auto mb-4 opacity-50" />
			<p class="text-sm">
				{searchQuery.trim() ? 'No documents found matching your search' : 'No documents yet'}
			</p>
		</div>
	{:else}
		<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
			{#each filteredDocuments as doc}
				<DocumentCard
					document={doc}
					onClick={() => onDocumentClick?.(doc)}
					onDelete={() => onDocumentDelete?.(doc)}
				/>
			{/each}
		</div>
	{/if}
</div>

