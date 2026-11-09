<!--
	Credential Vault - Main vault interface for managing encrypted credentials
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import {
    credentialActions,
    filteredCredentials,
    credentialStats,
    isLoading,
    error,
  } from '../stores/credentialStore';
  import { credentialService } from '../services/credentialService';
  import { logger } from '$lib/domains/shared';
  import { toast } from '$lib/utils/toast';
  import CredentialCard from './CredentialCard.svelte';
  import CredentialForm from './CredentialForm.svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Card, CardContent } from '$lib/components/ui/card';
  import Select from '$lib/components/ui/select.svelte';
  import Plus from '~icons/fa6-solid/plus';
  import Search from '~icons/fa6-solid/magnifying-glass';
  import Shield from '~icons/fa6-solid/shield';
  import Lock from '~icons/fa6-solid/lock';
  import { PageEmpty, PageError, PageHeader, PageLoading, PageStats } from '$lib/components/shell';
  import type { CredentialType, Credential } from '../types';
  import { createVaultStatusQuery } from '../queries/vaultQueries';
  import { vaultGate } from '../state/vault.svelte';
  import { lockVault } from '../api/vaultApi';
  import { queryClient } from '$lib/domains/shared/query/queryClient';
  import { queryKeys } from '$lib/domains/shared/query/keys';

  const vaultStatusQuery = createVaultStatusQuery();
  const vaultStatus = $derived(vaultStatusQuery.data);
  const vaultLocked = $derived(vaultStatus?.state === 'locked');

  async function handleLock() {
    await lockVault();
    void queryClient.invalidateQueries({ queryKey: queryKeys.credentials.status });
  }

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

  // Forced one-time migration prompt: secrets already exist but the vault was
  // never set up (pre-vault installs), so gate as soon as status resolves.
  let setupPromptShown = false;
  $effect(() => {
    if (!setupPromptShown && vaultStatus?.state === 'needs_setup') {
      setupPromptShown = true;
      void vaultGate.ensureUnlocked();
    }
  });

  async function loadCredentials() {
    try {
      credentialActions.setLoading(true);
      credentialActions.setError(null);

      logger.info('CredentialVault', 'Loading credentials');

      const credentialList = await credentialService.getCredentials();
      credentialActions.setCredentials(credentialList);

      logger.info('Credentials loaded', {
        context: 'CredentialVault',
        count: credentialList.length,
      });
    } catch (err) {
      logger.error('Failed to load credentials', {
        context: 'CredentialVault',
        error: err,
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

  async function handleDeleteCredential(credentialId: string) {
    try {
      await credentialService.deleteCredential(credentialId);
      credentialActions.removeCredential(credentialId);
      toast.success('Credential deleted');
    } catch (err) {
      toast.error(
        typeof err === 'string'
          ? err
          : err instanceof Error
            ? err.message
            : 'Failed to delete credential'
      );
    }
  }
</script>

<div class="space-y-6">
  <PageHeader
    title="Credential Vault"
    description="Securely manage your SSH keys, API tokens, and environment variables"
  >
    {#snippet actions()}
      {#if !vaultLocked}
        <Button variant="ghost" size="sm" onclick={handleLock}>
          <Lock class="mr-2 h-4 w-4" />
          Lock
        </Button>
      {/if}
      <Button onclick={handleCreateCredential}>
        <Plus class="mr-2 h-4 w-4" />
        Add Credential
      </Button>
    {/snippet}
  </PageHeader>

  {#if vaultLocked}
    <Card>
      <CardContent class="flex items-center justify-between gap-4 py-4">
        <div class="flex items-center gap-3">
          <Shield class="h-5 w-5 text-muted-foreground" />
          <span class="text-sm text-muted-foreground">
            Vault is locked — unlock it to reveal, copy, or decrypt credential values.
          </span>
        </div>
        <Button size="sm" onclick={() => vaultGate.ensureUnlocked()}>Unlock</Button>
      </CardContent>
    </Card>
  {/if}

  <PageStats
    stats={[
      { label: 'Active', value: stats.active, icon: Shield },
      { label: 'Expired', value: stats.expired },
      { label: 'Types', value: Object.keys(stats.byType).length },
    ]}
  />

  <!-- Filters -->
  <div class="flex flex-col gap-4 sm:flex-row">
    <div class="flex-1">
      <div class="relative">
        <Search
          class="absolute top-1/2 left-3 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
        />
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
        { value: 'other', label: 'Other' },
      ]}
      defaultValue={selectedType || ''}
      placeholder="Filter by type"
      onSelect={(value) => handleTypeFilter(value ? (value as CredentialType) : null)}
      class="w-[200px]"
    />
  </div>

  {#if loading}
    <PageLoading message="Loading credentials..." />
  {:else if errorMessage}
    <PageError
      title="Failed to load credentials"
      message={errorMessage}
      onRetry={loadCredentials}
    />
  {:else if credentialList.length === 0}
    <PageEmpty
      title="No credentials yet"
      description="Add your first credential to get started."
      filteredDescription="Try adjusting your search or filters."
      isFiltered={Boolean(searchQuery.trim() || selectedType)}
      icon={Shield}
      actionLabel="Add Credential"
      onAction={handleCreateCredential}
    />
  {:else}
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
</div>

<!-- Credential Form Modal -->
{#if showForm}
  <CredentialForm
    credential={editingCredential}
    onSave={handleFormSave}
    onClose={handleFormClose}
  />
{/if}
