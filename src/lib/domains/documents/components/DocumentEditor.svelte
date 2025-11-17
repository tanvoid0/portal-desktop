<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Textarea } from '$lib/components/ui/textarea';
	import { Label } from '$lib/components/ui/label';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { Separator } from '$lib/components/ui/separator';
	import Select from '$lib/components/ui/select.svelte';
	import Icon from '@iconify/svelte';
	import { documentActions } from '../stores/documentStore';
	import { toastActions } from '@/lib/domains/shared/stores/toastStore';
	import type { Document } from '../types';

	interface Props {
		document?: Document | null;
		onSave?: (doc: Document) => void;
		onCancel?: () => void;
		initialTitle?: string;
		initialContent?: string;
	}

	let { document, onSave, onCancel, initialTitle, initialContent }: Props = $props();

	// Form state
	let title = $state(document?.title || initialTitle || '');
	let content = $state(document?.content || initialContent || '');
	let contentDraft = $state(document?.contentDraft || '');
	let isArchived = $state(document?.isArchived || false);
	let tags = $state<string[]>(document?.tags || []);
	let isDraft = $state(document?.isDraft || false);

	// UI state
	let isSaving = $state(false);
	let newTag = $state('');
	let autoSaveTimer: ReturnType<typeof setTimeout> | null = null;

	const isCreateMode = $derived(document === null);

	// Auto-save draft
	function startAutoSave() {
		if (autoSaveTimer) {
			clearTimeout(autoSaveTimer);
		}
		autoSaveTimer = setTimeout(async () => {
			if (!isCreateMode && document) {
				try {
					await documentActions.updateDraft(document.id, contentDraft || content);
				} catch (error) {
					console.error('Auto-save failed:', error);
				}
			}
		}, 2000);
	}

	function addTag() {
		if (newTag.trim() && !tags.includes(newTag.trim())) {
			tags = [...tags, newTag.trim()];
			newTag = '';
		}
	}

	function removeTag(tagToRemove: string) {
		tags = tags.filter((tag) => tag !== tagToRemove);
	}

	async function handleSave() {
		if (!title.trim()) {
			toastActions.error('Document title is required');
			return;
		}

		try {
			isSaving = true;
			let savedDoc: Document;

			if (isCreateMode) {
				savedDoc = await documentActions.createDocument(title.trim(), content, tags, isArchived);
				toastActions.success('Document created successfully');
			} else {
				savedDoc = await documentActions.saveDocument(document!.id, title.trim(), content, tags, isArchived);
				toastActions.success('Document saved successfully');
			}

			onSave?.(savedDoc);
		} catch (error) {
			toastActions.error(
				`Failed to ${isCreateMode ? 'create' : 'save'} document`,
				error instanceof Error ? error.message : 'An unexpected error occurred'
			);
		} finally {
			isSaving = false;
		}
	}

	$effect(() => {
		if (contentDraft || content) {
			startAutoSave();
		}
		return () => {
			if (autoSaveTimer) {
				clearTimeout(autoSaveTimer);
			}
		};
	});
</script>

<Card class="max-w-4xl mx-auto">
	<CardHeader>
		<CardTitle>{isCreateMode ? 'Create New Document' : 'Edit Document'}</CardTitle>
	</CardHeader>
	<CardContent class="space-y-6">
		<!-- Title -->
		<div class="space-y-2">
			<Label for="title">Title *</Label>
			<Input id="title" bind:value={title} placeholder="Enter document title..." />
		</div>

		<!-- Archived Toggle -->
		<div class="flex items-center space-x-2">
			<input
				type="checkbox"
				id="isArchived"
				bind:checked={isArchived}
				class="h-4 w-4 rounded border-gray-300"
			/>
			<Label for="isArchived" class="cursor-pointer">Archived (soft delete)</Label>
		</div>

		<!-- Content -->
		<div class="space-y-2">
			<Label for="content">Content</Label>
			<Textarea
				id="content"
				bind:value={content}
				placeholder="Enter document content (Markdown supported)..."
				rows={15}
				oninput={() => {
					contentDraft = content;
					startAutoSave();
				}}
			/>
			{#if isDraft}
				<p class="text-xs text-muted-foreground">Draft saved</p>
			{/if}
		</div>

		<!-- Tags -->
		<div class="space-y-2">
			<Label>Tags</Label>
			<div class="flex gap-2">
				<Input
					placeholder="Add a tag..."
					bind:value={newTag}
					onkeydown={(e) => {
						if (e.key === 'Enter') {
							e.preventDefault();
							addTag();
						}
					}}
				/>
				<Button type="button" variant="outline" onclick={addTag}>
					<Icon icon="lucide:plus" class="w-4 h-4" />
				</Button>
			</div>
			{#if tags.length > 0}
				<div class="flex flex-wrap gap-2">
					{#each tags as tag}
						<Badge variant="secondary" class="flex items-center gap-1">
							{tag}
							<Button
								type="button"
								variant="ghost"
								size="sm"
								onclick={() => removeTag(tag)}
								class="h-4 w-4 p-0"
							>
								<Icon icon="lucide:x" class="w-3 h-3" />
							</Button>
						</Badge>
					{/each}
				</div>
			{/if}
		</div>

		<Separator />

		<!-- Actions -->
		<div class="flex justify-end gap-3">
			<Button variant="outline" onclick={onCancel} disabled={isSaving}>
				Cancel
			</Button>
			<Button onclick={handleSave} disabled={isSaving} loading={isSaving}>
				{isCreateMode ? 'Create Document' : 'Save Document'}
			</Button>
		</div>
	</CardContent>
</Card>

