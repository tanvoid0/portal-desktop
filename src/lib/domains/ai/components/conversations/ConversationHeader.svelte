<script lang="ts">
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import {
		AlertDialog,
		AlertDialogAction,
		AlertDialogCancel,
		AlertDialogContent,
		AlertDialogDescription,
		AlertDialogFooter,
		AlertDialogHeader,
		AlertDialogTitle,
		AlertDialogTrigger
	} from '$lib/components/ui/alert-dialog';
	import Icon from '@iconify/svelte';
	import ChatModelSelector from '../chat/ChatModelSelector.svelte';
	import type { Conversation } from '../../types/index.js';

	interface Props {
		conversation: Conversation;
		selectedModel?: string | null;
		onTitleChange?: (title: string) => void;
		onDelete?: () => void;
		onBack?: () => void;
		onModelChange?: (model: string) => void;
	}

	let {
		conversation,
		selectedModel = $bindable<string | null>(null),
		onTitleChange,
		onDelete,
		onBack,
		onModelChange
	}: Props = $props();
	
	let selectedProvider = $state(conversation.provider);

	let isEditing = $state(false);
	let editedTitle = $state(conversation.title);

	function handleSave() {
		if (editedTitle.trim() && onTitleChange) {
			onTitleChange(editedTitle.trim());
		}
		isEditing = false;
	}

	function handleCancel() {
		editedTitle = conversation.title;
		isEditing = false;
	}

	$effect(() => {
		editedTitle = conversation.title;
		selectedProvider = conversation.provider;
	});
</script>

<div class="flex items-center gap-4 p-4 border-b">
	{#if onBack}
		<Button variant="ghost" size="sm" onclick={onBack}>
			<Icon icon="lucide:arrow-left" class="h-4 w-4 mr-2" />
			Back
		</Button>
	{/if}
	<div class="flex-1">
		{#if isEditing}
			<div class="flex items-center gap-2">
				<Input
					bind:value={editedTitle}
					class="flex-1"
					onkeydown={(e) => {
						if (e.key === 'Enter') handleSave();
						if (e.key === 'Escape') handleCancel();
					}}
				/>
				<Button variant="ghost" size="sm" onclick={handleSave}>
					<Icon icon="lucide:check" class="h-4 w-4" />
				</Button>
				<Button variant="ghost" size="sm" onclick={handleCancel}>
					<Icon icon="lucide:x" class="h-4 w-4" />
				</Button>
			</div>
		{:else}
			<div class="flex items-center gap-2">
				<h2 class="text-lg font-semibold">{conversation.title}</h2>
				<Button variant="ghost" size="sm" onclick={() => (isEditing = true)}>
					<Icon icon="lucide:edit" class="h-4 w-4" />
				</Button>
			</div>
		{/if}
	</div>
	<div class="flex items-center gap-2">
		<ChatModelSelector bind:selectedProvider bind:selectedModel onModelChange={onModelChange} />
	</div>
	{#if onDelete}
		<AlertDialog>
			<AlertDialogTrigger>
				<Button variant="destructive" size="sm">
					<Icon icon="lucide:trash-2" class="h-4 w-4 mr-2" />
					Delete
				</Button>
			</AlertDialogTrigger>
			<AlertDialogContent>
				<AlertDialogHeader>
					<AlertDialogTitle>Delete Conversation</AlertDialogTitle>
					<AlertDialogDescription>
						Are you sure you want to delete "{conversation.title}"? This action cannot be undone.
					</AlertDialogDescription>
				</AlertDialogHeader>
				<AlertDialogFooter>
					<AlertDialogCancel>Cancel</AlertDialogCancel>
					<AlertDialogAction onclick={onDelete}>Delete</AlertDialogAction>
				</AlertDialogFooter>
			</AlertDialogContent>
		</AlertDialog>
	{/if}
</div>

