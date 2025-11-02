<!--
	Credential Form - Add/edit credential dialog
-->

<script lang="ts">
	import { credentialService } from '../services/credentialService';
	import { logger } from '$lib/domains/shared';
	import { toast } from '$lib/domains/shared/stores/toastStore';
	import type { Credential, CredentialCreateRequest, CredentialUpdateRequest, CredentialMetadata } from '../types';
	import { CredentialType } from '../types';
	import { Dialog, DialogContent, DialogDescription, DialogHeader, DialogTitle } from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Textarea } from '$lib/components/ui/textarea';
	import Select from '$lib/components/ui/select.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { Loader2, AlertCircle, Plus, X } from 'lucide-svelte';

	interface Props {
		credential?: Credential | null;
		onSave?: (credential: Credential) => void;
		onClose?: () => void;
	}

	const { credential, onSave, onClose }: Props = $props();

	let loading = $state(false);
	let error = $state<string | null>(null);
	let validationErrors = $state<string[]>([]);

	// Form state
	let name = $state(credential?.name || '');
	let type = $state<CredentialType>(credential?.type || CredentialType.OTHER);
	let description = $state(credential?.description || '');
	let value = $state('');
	let tags = $state<string[]>(credential?.tags || []);
	let newTag = $state('');
	let expiresAt = $state<string>('');
	let metadata = $state<Partial<CredentialMetadata>>({});

	// Credential type options
	const credentialTypes: { value: CredentialType; label: string; icon: string }[] = [
		{ value: CredentialType.SSH_KEY, label: 'SSH Key', icon: '🔑' },
		{ value: CredentialType.API_TOKEN, label: 'API Token', icon: '🎫' },
		{ value: CredentialType.ENV_VAR, label: 'Environment Variable', icon: '🌍' },
		{ value: CredentialType.DATABASE, label: 'Database', icon: '🗄️' },
		{ value: CredentialType.CLOUD_PROVIDER, label: 'Cloud Provider', icon: '☁️' },
		{ value: CredentialType.REGISTRY, label: 'Registry', icon: '📦' },
		{ value: CredentialType.OTHER, label: 'Other', icon: '🔐' }
	];

	// Initialize form with credential data
	$effect(() => {
		if (credential) {
			name = credential.name;
			type = credential.type;
			description = credential.description || '';
			tags = [...credential.tags];
			expiresAt = credential.expiresAt ? credential.expiresAt.toISOString().split('T')[0] : '';
			metadata = { ...credential.metadata };
		}
	});

	async function handleSubmit() {
		// Validate form
		validationErrors = [];
		
		if (!name.trim()) {
			validationErrors.push('Name is required');
		}
		
		if (!value.trim()) {
			validationErrors.push('Value is required');
		}

		if (expiresAt && new Date(expiresAt) < new Date()) {
			validationErrors.push('Expiration date must be in the future');
		}

		if (validationErrors.length > 0) {
			return;
		}

		try {
			loading = true;
			error = null;

			if (credential) {
				// Update existing credential
				logger.info('Updating credential', { 
					context: 'CredentialForm', 
					data: { credentialId: credential.id } 
				});

				const updateRequest: CredentialUpdateRequest = {
					name: name.trim(),
					description: description.trim() || undefined,
					tags: tags,
					value: value.trim(),
					expiresAt: expiresAt ? new Date(expiresAt) : undefined,
					metadata: Object.keys(metadata).length > 0 ? metadata : undefined
				};

				const updatedCredential = await credentialService.updateCredential(credential.id, updateRequest);
				onSave?.(updatedCredential);
				
				logger.info('Credential updated successfully', { 
					context: 'CredentialForm', 
					data: { credentialId: credential.id } 
				});
			} else {
				// Create new credential
				logger.info('Creating credential', { 
					context: 'CredentialForm', 
					data: { name, type } 
				});

				const createRequest: CredentialCreateRequest = {
					name: name.trim(),
					type,
					description: description.trim() || undefined,
					tags,
					value: value.trim(),
					expiresAt: expiresAt ? new Date(expiresAt) : undefined,
					metadata: Object.keys(metadata).length > 0 ? metadata : undefined
				};

				const newCredential = await credentialService.createCredential(createRequest);
				onSave?.(newCredential);
				
				logger.info('Credential created successfully', { 
					context: 'CredentialForm', 
					data: { credentialId: newCredential.id } 
				});
			}

			toast.success(credential ? 'Credential updated' : 'Credential created');
		} catch (err) {
			logger.error('Failed to save credential', {
				context: 'CredentialForm',
				error: err,
				data: { name, type }
			});
			error = err instanceof Error ? err.message : 'Failed to save credential';
		} finally {
			loading = false;
		}
	}

	function handleAddTag() {
		if (newTag.trim() && !tags.includes(newTag.trim())) {
			tags = [...tags, newTag.trim()];
			newTag = '';
		}
	}

	function handleRemoveTag(tagToRemove: string) {
		tags = tags.filter(tag => tag !== tagToRemove);
	}

	function handleKeyPress(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			handleAddTag();
		}
	}

</script>

<Dialog open={true} onOpenChange={(open) => !open && onClose?.()}>
	<DialogContent class="sm:max-w-md">
		<DialogHeader>
			<DialogTitle>
				{credential ? 'Edit Credential' : 'Add New Credential'}
			</DialogTitle>
			<DialogDescription>
				{credential ? 'Update the credential details' : 'Add a new credential to your vault'}
			</DialogDescription>
		</DialogHeader>

		<div class="space-y-4">
			<!-- Name -->
			<div class="space-y-2">
				<Label for="name">Name *</Label>
				<Input
					id="name"
					placeholder="e.g., GitHub API Token"
					bind:value={name}
					disabled={loading}
				/>
			</div>

			<!-- Type -->
			<div class="space-y-2">
				<Label for="type">Type *</Label>
				<Select 
					options={credentialTypes.map(typeOption => ({
						value: typeOption.value,
						label: `${typeOption.icon} ${typeOption.label}`
					}))}
					defaultValue={type}
					placeholder="Select credential type"
					onSelect={(value) => type = value as CredentialType}
					disabled={loading}
				/>
			</div>

			<!-- Description -->
			<div class="space-y-2">
				<Label for="description">Description</Label>
				<Textarea
					id="description"
					placeholder="Optional description"
					bind:value={description}
					disabled={loading}
					rows={2}
				/>
			</div>

			<!-- Value -->
			<div class="space-y-2">
				<Label for="value">Value *</Label>
				<Textarea
					id="value"
					placeholder="Enter the credential value"
					bind:value={value}
					disabled={loading}
					rows={3}
					class="font-mono"
				/>
			</div>

			<!-- Tags -->
			<div class="space-y-2">
				<Label>Tags</Label>
				<div class="flex gap-2">
					<Input
						placeholder="Add tag"
						bind:value={newTag}
						onkeypress={handleKeyPress}
						disabled={loading}
					/>
					<Button type="button" variant="outline" onclick={handleAddTag} disabled={loading || !newTag.trim()}>
						<Plus class="h-4 w-4" />
					</Button>
				</div>
				{#if tags.length > 0}
					<div class="flex flex-wrap gap-1">
						{#each tags as tag (tag)}
							<Badge variant="secondary" class="flex items-center gap-1">
								{tag}
								<X class="h-3 w-3 cursor-pointer" onclick={() => handleRemoveTag(tag)} />
							</Badge>
						{/each}
					</div>
				{/if}
			</div>

			<!-- Expiration Date -->
			<div class="space-y-2">
				<Label for="expiresAt">Expiration Date</Label>
				<Input
					id="expiresAt"
					type="date"
					bind:value={expiresAt}
					disabled={loading}
				/>
			</div>

			<!-- Validation Errors -->
			{#if validationErrors.length > 0}
				<Alert variant="destructive">
					<AlertCircle class="h-4 w-4" />
					<AlertDescription>
						<ul class="list-disc list-inside">
							{#each validationErrors as validationError (validationError)}
								<li>{validationError}</li>
							{/each}
						</ul>
					</AlertDescription>
				</Alert>
			{/if}

			<!-- Error Alert -->
			{#if error}
				<Alert variant="destructive">
					<AlertCircle class="h-4 w-4" />
					<AlertDescription>
						{error}
					</AlertDescription>
				</Alert>
			{/if}

			<!-- Actions -->
			<div class="flex justify-end gap-2">
				<Button variant="outline" onclick={onClose} disabled={loading}>
					Cancel
				</Button>
				<Button onclick={handleSubmit} disabled={loading}>
					{#if loading}
						<Loader2 class="h-4 w-4 mr-2 animate-spin" />
					{/if}
					{credential ? 'Update' : 'Create'}
				</Button>
			</div>
		</div>
	</DialogContent>
</Dialog>
