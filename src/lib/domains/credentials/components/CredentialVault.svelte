<!--
	Credential Vault - Main vault interface for managing encrypted credentials
-->

<script lang="ts">
	import { onMount } from 'svelte';
	import { credentialActions, filteredCredentials, credentialStats, isLoading, error } from '../stores/credentialStore';
	import { credentialService } from '../services/credentialService';
	import { logger } from '$lib/domains/shared';
	import { toast } from '$lib/domains/shared/stores/toastStore';
	import CredentialCard from './CredentialCard.svelte';
	import CredentialForm from './CredentialForm.svelte';
	import { Button } from '$lib/components/ui/button';
	import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
	import { Input } from '$lib/components/ui/input';
	import Select from '$lib/components/ui/select.svelte';
	import { Badge } from '$lib/components/ui/badge';
	import { Alert, AlertDescription } from '$lib/components/ui/alert';
	import { Plus, Search, Shield, AlertCircle, Loader2 } from '@lucide/svelte';
	import type { CredentialType, Credential } from '../types';

	let searchQuery = $state('');
	let selectedType = $state<CredentialType | null>(null);
	let selectedTags = $state<string[]>([]);
	let showForm = $state(false);
	let editingCredential = $state<Credential | null>(null);

	// Reactive stores
	let credentialList = $derived($filteredCredentials);
	let stats = $derived($credentialStats);
	let loading = $derived($isLoading);
	let errorMessage = $derived($error);

	onMount(async () => {
		await loadCredentials();
	});

	async function loadCredentials() {
		try {
			credentialActions.setLoading(true);
			credentialActions.setError(null);
			
			logger.info('CredentialVault', 'Loading credentials');
			
			const credentialList = await credentialService.getCredentials();
			credentialActions.setCredentials(credentialList);
			
			logger.info('Credentials loaded', { context: 'CredentialVault', count: credentialList.length });
		} catch (err) {
			logger.error('Failed to load credentials', { context: 'CredentialVault', 
				error: err
			});
			credentialActions.setError(err instanceof Error ? err.message : 'Failed to load credentials');
			toast.error('Failed to load credentials');
		} finally {
			credentialActions.setLoading(false);
		}
	}

	function handleSearch() {
		credentialActions.setSearchQuery(searchQuery);
	}

	function handleTypeFilter(type: CredentialType | null) {
		selectedType = type;
		credentialActions.setSelectedType(type);
	}

	function handleTagFilter(tags: string[]) {
		selectedTags = tags;
		credentialActions.setSelectedTags(tags);
	}

	function handleCreateCredential() {
		editingCredential = null;
		showForm = true;
	}

	function handleEditCredential(credential: Credential) {
		editingCredential = credential;
		showForm = true;
	}

	function handleFormClose() {
		showForm = false;
		editingCredential = null;
	}

	function handleFormSave(credential: Credential) {
		if (editingCredential) {
			credentialActions.updateCredential(credential.id, credential);
			toast.success('Credential updated');
		} else {
			credentialActions.addCredential(credential);
			toast.success('Credential created');
		}
		handleFormClose();
	}

	function handleDeleteCredential(credentialId: string) {
		credentialActions.removeCredential(credentialId);
		toast.success('Credential deleted');
	}

</script>

<div class="space-y-6">
	<!-- Header -->
	<div class="flex items-center justify-between">
		<div>
			<h1 class="text-3xl font-bold tracking-tight flex items-center gap-2">
				<Shield class="h-8 w-8" />
				Credential Vault
			</h1>
			<p class="text-muted-foreground">
				Securely manage your SSH keys, API tokens, and environment variables
			</p>
		</div>
		<Button onclick={handleCreateCredential}>
			<Plus class="h-4 w-4 mr-2" />
			Add Credential
		</Button>
	</div>

	<!-- Error Alert -->
	{#if errorMessage}
		<Alert variant="destructive">
			<AlertCircle class="h-4 w-4" />
			<AlertDescription>
				{errorMessage}
			</AlertDescription>
		</Alert>
	{/if}

	<!-- Stats Cards -->
	<div class="grid gap-4 md:grid-cols-4">
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Total Credentials</CardTitle>
				<Shield class="h-4 w-4 text-muted-foreground" />
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{stats.total}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Active</CardTitle>
				<Badge variant="default">{stats.active}</Badge>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-green-600">{stats.active}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Expired</CardTitle>
				<Badge variant="destructive">{stats.expired}</Badge>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold text-red-600">{stats.expired}</div>
			</CardContent>
		</Card>
		<Card>
			<CardHeader class="flex flex-row items-center justify-between space-y-0 pb-2">
				<CardTitle class="text-sm font-medium">Types</CardTitle>
				<Badge variant="outline">{Object.keys(stats.byType).length}</Badge>
			</CardHeader>
			<CardContent>
				<div class="text-2xl font-bold">{Object.keys(stats.byType).length}</div>
			</CardContent>
		</Card>
	</div>

	<!-- Filters -->
	<div class="flex flex-col sm:flex-row gap-4">
		<div class="flex-1">
			<div class="relative">
				<Search class="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
				<Input
					placeholder="Search credentials..."
					bind:value={searchQuery}
					oninput={handleSearch}
					class="pl-10"
				/>
			</div>
		</div>
		<Select 
			options={[
				{ value: '', label: 'All Types' },
				{ value: 'ssh_key', label: 'SSH Keys' },
				{ value: 'api_token', label: 'API Tokens' },
				{ value: 'env_var', label: 'Environment Variables' },
				{ value: 'database', label: 'Database' },
				{ value: 'cloud_provider', label: 'Cloud Provider' },
				{ value: 'registry', label: 'Registry' },
				{ value: 'other', label: 'Other' }
			]}
			defaultValue={selectedType || ''}
			placeholder="Filter by type"
			onSelect={(value) => handleTypeFilter(value ? value as CredentialType : null)}
			class="w-[200px]"
		/>
	</div>

	<!-- Loading State -->
	{#if loading}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each Array(6) as _, index (index)}
				<Card>
					<CardHeader>
						<div class="h-4 bg-muted animate-pulse rounded"></div>
						<div class="h-3 bg-muted animate-pulse rounded w-2/3"></div>
					</CardHeader>
					<CardContent>
						<div class="space-y-2">
							<div class="h-3 bg-muted animate-pulse rounded"></div>
							<div class="h-3 bg-muted animate-pulse rounded w-1/2"></div>
						</div>
					</CardContent>
				</Card>
			{/each}
		</div>
	{/if}

	<!-- Credentials Grid -->
	{#if !loading && credentialList.length > 0}
		<div class="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
			{#each credentialList as credential (credential.id)}
				<CredentialCard 
					{credential}
					onEdit={() => handleEditCredential(credential)}
					onDelete={() => handleDeleteCredential(credential.id)}
				/>
			{/each}
		</div>
	{/if}

	<!-- Empty State -->
	{#if !loading && credentialList.length === 0}
		<Card>
			<CardContent class="flex flex-col items-center justify-center py-12">
				<Shield class="h-12 w-12 text-muted-foreground mb-4" />
				<h3 class="text-lg font-semibold mb-2">No Credentials Found</h3>
				<p class="text-muted-foreground text-center mb-4">
					{searchQuery || selectedType ? 'Try adjusting your filters' : 'Add your first credential to get started'}
				</p>
				{#if !searchQuery && !selectedType}
					<Button onclick={handleCreateCredential}>
						<Plus class="h-4 w-4 mr-2" />
						Add Credential
					</Button>
				{/if}
			</CardContent>
		</Card>
	{/if}
</div>

<!-- Credential Form Modal -->
{#if showForm}
	<CredentialForm 
		credential={editingCredential}
		onSave={handleFormSave}
		onClose={handleFormClose}
	/>
{/if}
