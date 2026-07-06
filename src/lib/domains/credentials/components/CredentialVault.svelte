<!--
	Credential Vault - Main vault interface for managing encrypted credentials
-->

<script lang="ts">
  import { onMount } from "svelte";
  import {
    credentialActions,
    filteredCredentials,
    credentialStats,
    isLoading,
    error,
  } from "../stores/credentialStore";
  import { credentialService } from "../services/credentialService";
  import { logger } from "$lib/domains/shared";
  import { toast } from "$lib/utils/toast";
  import CredentialCard from "./CredentialCard.svelte";
  import CredentialForm from "./CredentialForm.svelte";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import Select from "$lib/components/ui/select.svelte";
  import { Badge } from "$lib/components/ui/badge";
  import { Plus, Search, Shield } from "@lucide/svelte";
  import {
    PageHeader,
    PageLoading,
    PageError,
    PageEmpty,
  } from "$lib/components/shell";
  import type { CredentialType, Credential } from "../types";

  let searchQuery = $state("");
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

      logger.info("CredentialVault", "Loading credentials");

      const credentialList = await credentialService.getCredentials();
      credentialActions.setCredentials(credentialList);

      logger.info("Credentials loaded", {
        context: "CredentialVault",
        count: credentialList.length,
      });
    } catch (err) {
      logger.error("Failed to load credentials", {
        context: "CredentialVault",
        error: err,
      });
      credentialActions.setError(
        err instanceof Error ? err.message : "Failed to load credentials",
      );
      toast.error("Failed to load credentials");
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
      toast.success("Credential updated");
    } else {
      credentialActions.addCredential(credential);
      toast.success("Credential created");
    }
    handleFormClose();
  }

  function handleDeleteCredential(credentialId: string) {
    credentialActions.removeCredential(credentialId);
    toast.success("Credential deleted");
  }
</script>

<div class="container mx-auto space-y-6 p-6">
  <PageHeader
    title="Credential Vault"
    description="Securely manage your SSH keys, API tokens, and environment variables"
  >
    {#snippet actions()}
      <Button onclick={handleCreateCredential}>
        <Plus class="mr-2 h-4 w-4" />
        Add Credential
      </Button>
    {/snippet}
  </PageHeader>

  <!-- Stats Cards -->
  <div class="grid gap-4 md:grid-cols-4">
    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">Total Credentials</CardTitle>
        <Shield class="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold">{stats.total}</div>
      </CardContent>
    </Card>
    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">Active</CardTitle>
        <Badge variant="default">{stats.active}</Badge>
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold text-green-600">{stats.active}</div>
      </CardContent>
    </Card>
    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">Expired</CardTitle>
        <Badge variant="destructive">{stats.expired}</Badge>
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold text-red-600">{stats.expired}</div>
      </CardContent>
    </Card>
    <Card>
      <CardHeader
        class="flex flex-row items-center justify-between space-y-0 pb-2"
      >
        <CardTitle class="text-sm font-medium">Types</CardTitle>
        <Badge variant="outline">{Object.keys(stats.byType).length}</Badge>
      </CardHeader>
      <CardContent>
        <div class="text-2xl font-bold">{Object.keys(stats.byType).length}</div>
      </CardContent>
    </Card>
  </div>

  <!-- Filters -->
  <div class="flex flex-col gap-4 sm:flex-row">
    <div class="flex-1">
      <div class="relative">
        <Search
          class="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 transform text-muted-foreground"
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
        { value: "", label: "All Types" },
        { value: "ssh_key", label: "SSH Keys" },
        { value: "api_token", label: "API Tokens" },
        { value: "env_var", label: "Environment Variables" },
        { value: "database", label: "Database" },
        { value: "cloud_provider", label: "Cloud Provider" },
        { value: "registry", label: "Registry" },
        { value: "other", label: "Other" },
      ]}
      defaultValue={selectedType || ""}
      placeholder="Filter by type"
      onSelect={(value) =>
        handleTypeFilter(value ? (value as CredentialType) : null)}
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
