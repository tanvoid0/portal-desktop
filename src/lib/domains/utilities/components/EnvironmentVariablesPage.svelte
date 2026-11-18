<script lang="ts">
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertDescription, AlertTitle } from '$lib/components/ui/alert';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import Save from '~icons/fa6-solid/floppy-disk';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Shield from '~icons/fa6-solid/shield';
  import ShieldAlert from '~icons/fa6-solid/shield-halved';
  import Loader2 from '~icons/fa6-solid/spinner';
  import Variable from '~icons/fa6-solid/square-root-variable';
  import User from '~icons/fa6-solid/user';
  import Monitor from '~icons/fa6-solid/desktop';
  import EnvVariableRow from './EnvVariableRow.svelte';
  import EnvScopeCard from './EnvScopeCard.svelte';
  import {
    EnvironmentVariablesState,
    type EditableScope,
  } from '../state/environmentVariables.svelte';
  import type { EnvRow } from '../types';

  const st = new EnvironmentVariablesState();

  onMount(st.load);
</script>

{#snippet variableList(sectionRows: EnvRow[], search: string)}
  {#if sectionRows.length === 0}
    <p class="py-8 text-center text-sm text-muted-foreground">
      {search.trim()
        ? `No variables match "${search.trim()}".`
        : 'No variables in this section yet.'}
    </p>
  {:else}
    {#each sectionRows as row (row.id)}
      <EnvVariableRow
        {row}
        expanded={st.expandedValues.has(row.id)}
        ontoggle={() => st.toggleExpanded(row.id)}
        onupdate={(patch) => st.updateRow(row.id, patch)}
        onremove={() => st.removeRow(row.id)}
      />
    {/each}
  {/if}
{/snippet}

<div class="space-y-6">
  <header>
    <div class="mb-2 flex items-center gap-2.5">
      <div
        class="flex h-6 w-6 items-center justify-center rounded-md bg-primary text-primary-foreground"
      >
        <Variable class="size-3.5" />
      </div>
      <h1 class="text-lg font-semibold tracking-tight text-foreground">Environment Variables</h1>
    </div>
    <p class="max-w-2xl text-sm text-muted-foreground">
      User variables apply to your account. System variables apply machine-wide and may require
      administrator approval — Portal stays open and refreshes values after you approve.
    </p>
  </header>

  {#if st.loading}
    <div class="flex items-center gap-2 text-sm text-muted-foreground">
      <Loader2 class="h-4 w-4 animate-spin" />
      Loading environment variables…
    </div>
  {:else if st.error}
    <Alert variant="destructive">
      <AlertTitle>Unable to load</AlertTitle>
      <AlertDescription>{st.error}</AlertDescription>
    </Alert>
  {:else}
    {#if st.permissions}
      <div class="flex flex-wrap items-center gap-2">
        <Badge variant="outline">Platform: {st.permissions.platform}</Badge>
        {#if st.permissions.isElevated}
          <Badge variant="default" class="gap-1">
            <Shield class="h-3 w-3" />
            Elevated
          </Badge>
        {:else}
          <Badge variant="secondary" class="gap-1">
            <ShieldAlert class="h-3 w-3" />
            Standard privileges
          </Badge>
        {/if}
        {#if st.dirtyCount > 0}
          <Badge variant="outline">{st.dirtyCount} unsaved change(s)</Badge>
        {/if}
      </div>
    {/if}

    <Tabs value={st.tab} onValueChange={(v) => (st.tab = v as EditableScope)} class="space-y-4">
      <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <TabsList>
          <TabsTrigger value="user" class="gap-2">
            <User class="h-4 w-4" />
            User
            {#if st.userDirtyCount > 0}
              <Badge variant="secondary" class="ml-1 h-5 min-w-5 px-1">
                {st.userDirtyCount}
              </Badge>
            {/if}
          </TabsTrigger>
          <TabsTrigger value="system" class="gap-2">
            <Monitor class="h-4 w-4" />
            System
            {#if st.systemDirtyCount > 0}
              <Badge variant="secondary" class="ml-1 h-5 min-w-5 px-1">
                {st.systemDirtyCount}
              </Badge>
            {/if}
          </TabsTrigger>
        </TabsList>

        <Button variant="outline" onclick={st.load} disabled={st.saving}>
          <RefreshCw class="mr-2 h-4 w-4" />
          Refresh
        </Button>
      </div>

      <TabsContent value="user">
        <EnvScopeCard
          title="User variables"
          description="Persistent for your account / user profile. Changes apply immediately without elevation."
          bind:search={st.userSearch}
          searchPlaceholder="Search user variables…"
          addDisabled={st.saving || st.permissions?.canEditUser === false}
          onadd={() => st.addRow('user')}
        >
          {@render variableList(st.userRows, st.userSearch)}
        </EnvScopeCard>
      </TabsContent>

      <TabsContent value="system">
        <EnvScopeCard
          title="System variables"
          description="Machine-wide variables visible to all users and services."
          bind:search={st.systemSearch}
          searchPlaceholder="Search system variables…"
          addDisabled={st.saving}
          onadd={() => st.addRow('system')}
        >
          {#if st.permissions && !st.permissions.canEditSystem}
            <Alert>
              <ShieldAlert class="h-4 w-4" />
              <AlertTitle>Administrator approval required</AlertTitle>
              <AlertDescription>
                Saving changes in this tab will open a UAC / administrator prompt. Portal stays open
                and picks up the new values when you approve.
              </AlertDescription>
            </Alert>
          {/if}
          {@render variableList(st.systemRows, st.systemSearch)}
        </EnvScopeCard>
      </TabsContent>
    </Tabs>

    <div class="flex flex-wrap gap-2">
      <Button onclick={st.save} disabled={st.saving || st.dirtyCount === 0}>
        {#if st.saving}
          <Loader2 class="mr-2 h-4 w-4 animate-spin" />
        {:else}
          <Save class="mr-2 h-4 w-4" />
        {/if}
        {st.needsElevation ? 'Save with elevation' : 'Save all changes'}
      </Button>
      <Button
        variant="outline"
        onclick={st.discardChanges}
        disabled={st.saving || st.dirtyCount === 0}
      >
        Discard
      </Button>
    </div>
  {/if}
</div>
