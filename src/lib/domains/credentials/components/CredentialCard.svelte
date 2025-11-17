<!--
	Credential Card - Display individual credential with masked value
-->

<script lang="ts">
	import { credentialService } from '../services/credentialService';
	import { logger } from '$lib/domains/shared';
	import { toast } from '$lib/domains/shared/stores/toastStore';
	import type { Credential, CredentialType } from '../types';
	import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
	import { Eye, EyeOff, Copy, Edit, Trash2, Calendar, Tag } from '@lucide/svelte';

	interface Props {
		credential: Credential;
		onEdit?: (credential: Credential) => void;
		onDelete?: (credentialId: string) => void;
	}

	const { credential, onEdit, onDelete }: Props = $props();

	let showValue = $state(false);
	let decryptedValue = $state<string | null>(null);
	let isDecrypting = $state(false);
	let showDeleteDialog = $state(false);

	async function handleToggleVisibility() {
		if (showValue && decryptedValue) {
			showValue = false;
			decryptedValue = null;
			return;
		}

		try {
			isDecrypting = true;
			
			logger.info('Decrypting credential', { 
				context: 'CredentialCard', 
				data: { credentialId: credential.id } 
			});

			const value = await credentialService.decryptCredential(credential.id);
			decryptedValue = value;
			showValue = true;
			
			logger.info('Credential decrypted successfully', { 
				context: 'CredentialCard', 
				data: { credentialId: credential.id } 
			});
		} catch (err) {
			logger.error('Failed to decrypt credential', {
				context: 'CredentialCard',
				error: err,
				data: { credentialId: credential.id }
			});
			toast.error('Failed to decrypt credential');
		} finally {
			isDecrypting = false;
		}
	}

	async function handleCopyValue() {
		if (!decryptedValue) return;

		try {
			await navigator.clipboard.writeText(decryptedValue);
			toast.success('Value copied to clipboard');
		} catch (err) {
			logger.error('Failed to copy to clipboard', {
				context: 'CredentialCard',
				error: err
			});
			toast.error('Failed to copy to clipboard');
		}
	}

	function handleEdit() {
		onEdit?.(credential);
	}

	function handleDelete() {
		showDeleteDialog = true;
	}

	function confirmDelete() {
		onDelete?.(credential.id);
		showDeleteDialog = false;
	}

	function getStatusColor(status: string): string {
		const colors: Record<string, string> = {
			active: 'bg-green-100 text-green-800',
			inactive: 'bg-gray-100 text-gray-800',
			expired: 'bg-red-100 text-red-800',
			revoked: 'bg-orange-100 text-orange-800'
		};
		return colors[status] || 'bg-gray-100 text-gray-800';
	}

	function getTypeIcon(type: CredentialType): string {
		return credentialService.getCredentialTypeIcon(type);
	}

	function getTypeColor(type: CredentialType): string {
		return credentialService.getCredentialTypeColor(type);
	}

	function formatDate(date: Date): string {
		return new Intl.DateTimeFormat('en-US', {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		}).format(date);
	}

	function isExpired(): boolean {
		return credential.expiresAt ? credential.expiresAt < new Date() : false;
	}
</script>

<Card class="hover:shadow-md transition-shadow">
	<CardHeader>
		<div class="flex items-start justify-between">
			<div class="flex items-center gap-2">
				<span class="text-2xl">{getTypeIcon(credential.type)}</span>
				<div>
					<CardTitle class="text-lg">{credential.name}</CardTitle>
					<CardDescription class="flex items-center gap-2">
						<Badge variant="outline" class={getTypeColor(credential.type)}>
							{credential.type.replace('_', ' ')}
						</Badge>
						<Badge variant={isExpired() ? 'destructive' : 'secondary'} class={getStatusColor(credential.status)}>
							{credential.status}
						</Badge>
					</CardDescription>
				</div>
			</div>
			<div class="flex items-center gap-1">
				<Button variant="ghost" size="sm" onclick={handleToggleVisibility} disabled={isDecrypting}>
					{#if isDecrypting}
						<Eye class="h-4 w-4 animate-pulse" />
					{:else if showValue}
						<EyeOff class="h-4 w-4" />
					{:else}
						<Eye class="h-4 w-4" />
					{/if}
				</Button>
				<Button variant="ghost" size="sm" onclick={handleEdit}>
					<Edit class="h-4 w-4" />
				</Button>
				<Button variant="ghost" size="sm" onclick={handleDelete}>
					<Trash2 class="h-4 w-4" />
				</Button>
			</div>
		</div>
	</CardHeader>
	
	<CardContent>
		<div class="space-y-3">
			<!-- Description -->
			{#if credential.description}
				<p class="text-sm text-muted-foreground">{credential.description}</p>
			{/if}

			<!-- Value Display -->
			<div class="space-y-2">
				<div class="flex items-center justify-between">
					<span class="text-sm font-medium">Value</span>
					{#if showValue && decryptedValue}
						<Button variant="ghost" size="sm" onclick={handleCopyValue}>
							<Copy class="h-3 w-3 mr-1" />
							Copy
						</Button>
					{/if}
				</div>
				<div class="p-2 bg-muted rounded-md font-mono text-sm">
					{#if showValue && decryptedValue}
						{decryptedValue}
					{:else}
						{credentialService.maskValue('encrypted_value', credential.type)}
					{/if}
				</div>
			</div>

			<!-- Tags -->
			{#if credential.tags.length > 0}
				<div class="flex items-center gap-2">
					<Tag class="h-3 w-3 text-muted-foreground" />
					<div class="flex flex-wrap gap-1">
						{#each credential.tags as tag (tag)}
							<Badge variant="outline" class="text-xs">{tag}</Badge>
						{/each}
					</div>
				</div>
			{/if}

			<!-- Metadata -->
			<div class="space-y-1 text-xs text-muted-foreground">
				{#if credential.lastUsed}
					<div class="flex items-center gap-1">
						<Calendar class="h-3 w-3" />
						Last used: {formatDate(credential.lastUsed)}
					</div>
				{/if}
				{#if credential.expiresAt}
					<div class="flex items-center gap-1" class:text-red-500={isExpired()}>
						<Calendar class="h-3 w-3" />
						Expires: {formatDate(credential.expiresAt)}
					</div>
				{/if}
			</div>
		</div>
	</CardContent>
</Card>

<!-- Delete Confirmation Dialog -->
<Dialog bind:open={showDeleteDialog}>
	<DialogContent>
		<DialogHeader>
			<DialogTitle>Delete Credential</DialogTitle>
			<DialogDescription>
				Are you sure you want to delete "{credential.name}"? This action cannot be undone.
			</DialogDescription>
		</DialogHeader>
		<div class="flex justify-end gap-2">
			<Button variant="outline" onclick={() => showDeleteDialog = false}>
				Cancel
			</Button>
			<Button variant="destructive" onclick={confirmDelete}>
				Delete
			</Button>
		</div>
	</DialogContent>
</Dialog>
