<script lang="ts">
  import { onMount, tick } from 'svelte';
  import { beforeNavigate, goto } from '$app/navigation';
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
  import { toast } from '$lib/utils/toast';
  import { confirmAction } from '$lib/utils/confirm';
  import { isTauriEnvironment } from '$lib/utils/tauri';
  import EnvVariableRow from './EnvVariableRow.svelte';
  import EnvScopeCard from './EnvScopeCard.svelte';
  import { hasMultipleSegments } from '../utils/envSegments';
  import type { EnvPermissions, EnvRow, EnvVariable } from '../types';
  import {
    applyEnvironmentChanges,
    buildChanges,
    getEnvironmentPermissions,
    hasSystemChanges,
    listEnvironmentVariables,
    refreshProcessEnvironment,
  } from '../services/environmentService';

  type EditableScope = 'user' | 'system';

  let loading = $state(true);
  let saving = $state(false);
  let error = $state<string | null>(null);
  let tab = $state<EditableScope>('user');
  let userSearch = $state('');
  let systemSearch = $state('');
  let permissions = $state<EnvPermissions | null>(null);
  let original = $state<EnvVariable[]>([]);
  let rows = $state<EnvRow[]>([]);
  let nextId = 1;
  let expandedValues = $state<Set<string>>(new Set());

  function toggleExpanded(id: string) {
    const next = new Set(expandedValues);
    if (next.has(id)) next.delete(id);
    else next.add(id);
    expandedValues = next;
  }

  function matchesSearch(row: EnvRow, query: string): boolean {
    const q = query.trim().toLowerCase();
    if (!q) return true;
    return row.name.toLowerCase().includes(q) || row.value.toLowerCase().includes(q);
  }

  function rowsForScope(scope: EditableScope): EnvRow[] {
    return rows.filter((row) => {
      if (row.isDeleted) return false;
      if (scope === 'user') {
        return row.scope === 'user' || row.scope === 'session';
      }
      return row.scope === 'system';
    });
  }

  const userRows = $derived(rowsForScope('user').filter((row) => matchesSearch(row, userSearch)));

  const systemRows = $derived(
    rowsForScope('system').filter((row) => matchesSearch(row, systemSearch))
  );

  const dirtyCount = $derived(rows.filter((r) => r.isDirty || r.isNew || r.isDeleted).length);

  const userDirtyCount = $derived.by(() => {
    const changes = buildChanges(original, rows);
    return changes.filter((c) => c.scope === 'user' || c.scope === 'session').length;
  });

  const systemDirtyCount = $derived.by(() => {
    const changes = buildChanges(original, rows);
    return changes.filter((c) => c.scope === 'system').length;
  });

  const needsElevation = $derived.by(() => {
    if (!permissions) return false;
    const changes = buildChanges(original, rows);
    return hasSystemChanges(changes) && !permissions.canEditSystem;
  });

  onMount(() => {
    load();
  });

  let leaveConfirmed = false;
  beforeNavigate((nav) => {
    const { to, willUnload, cancel } = nav;
    if (leaveConfirmed || willUnload || !to?.url || to.url.origin !== location.origin) return;
    if (buildChanges(original, rows).length === 0) return;
    cancel();
    confirmAction('You have unsaved changes. Leave and discard them?', 'Unsaved changes', {
      confirmLabel: 'Leave',
      destructive: true,
    }).then((ok) => {
      if (!ok) return;
      leaveConfirmed = true;
      if (nav.type === 'popstate') history.go(nav.delta);
      else goto(to.url);
    });
  });

  async function load() {
    if (!isTauriEnvironment()) {
      loading = false;
      error = 'Environment variable editing is only available in the desktop app.';
      return;
    }

    loading = true;
    error = null;
    try {
      const [vars, perms] = await Promise.all([
        listEnvironmentVariables(),
        getEnvironmentPermissions(),
      ]);
      original = vars;
      permissions = perms;
      rows = vars.map((v) => ({
        id: `row-${nextId++}`,
        name: v.name,
        value: v.value,
        scope: v.scope,
      }));
      expandedValues = new Set(rows.filter((r) => hasMultipleSegments(r.value)).map((r) => r.id));
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to load environment variables';
    } finally {
      loading = false;
    }
  }

  async function addRow(scope: EditableScope) {
    tab = scope;
    // an active search would hide the empty new row
    if (scope === 'user') userSearch = '';
    else systemSearch = '';
    const id = `row-${nextId++}`;
    rows = [
      ...rows,
      {
        id,
        name: '',
        value: '',
        scope,
        isNew: true,
        isDirty: true,
      },
    ];
    await tick();
    document.getElementById(`name-${id}`)?.focus();
  }

  function updateRow(id: string, patch: Partial<EnvRow>) {
    rows = rows.map((row) => {
      if (row.id !== id) return row;
      const updated = { ...row, ...patch, isDirty: true };
      if (!row.isNew) updated.isDirty = true;
      return updated;
    });
  }

  function removeRow(id: string) {
    rows = rows.map((row) => {
      if (row.id !== id) return row;
      if (row.isNew) return { ...row, isDeleted: true };
      return { ...row, isDeleted: true, isDirty: true };
    });
  }

  function discardChanges() {
    rows = original.map((v) => ({
      id: `row-${nextId++}`,
      name: v.name,
      value: v.value,
      scope: v.scope,
    }));
    toast.info('Changes discarded');
  }

  async function save() {
    const changes = buildChanges(original, rows);
    if (changes.length === 0) {
      toast.info('No changes to save');
      return;
    }

    const invalid = rows.some((r) => !r.isDeleted && (r.isNew || r.isDirty) && !r.name.trim());
    if (invalid) {
      toast.error('All variables need a name before saving');
      return;
    }

    saving = true;
    try {
      const result = await applyEnvironmentChanges(changes);
      if (result.success) {
        toast.success(result.message);
        await refreshProcessEnvironment();
        await load();
      } else {
        toast.warning(result.message);
      }
    } catch (err) {
      toast.error(err instanceof Error ? err.message : 'Failed to save environment variables');
    } finally {
      saving = false;
    }
  }
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
        expanded={expandedValues.has(row.id)}
        ontoggle={() => toggleExpanded(row.id)}
        onupdate={(patch) => updateRow(row.id, patch)}
        onremove={() => removeRow(row.id)}
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

  {#if loading}
    <div class="flex items-center gap-2 text-sm text-muted-foreground">
      <Loader2 class="h-4 w-4 animate-spin" />
      Loading environment variables…
    </div>
  {:else if error}
    <Alert variant="destructive">
      <AlertTitle>Unable to load</AlertTitle>
      <AlertDescription>{error}</AlertDescription>
    </Alert>
  {:else}
    {#if permissions}
      <div class="flex flex-wrap items-center gap-2">
        <Badge variant="outline">Platform: {permissions.platform}</Badge>
        {#if permissions.isElevated}
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
        {#if dirtyCount > 0}
          <Badge variant="outline">{dirtyCount} unsaved change(s)</Badge>
        {/if}
      </div>
    {/if}

    <Tabs value={tab} onValueChange={(v) => (tab = v as EditableScope)} class="space-y-4">
      <div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
        <TabsList>
          <TabsTrigger value="user" class="gap-2">
            <User class="h-4 w-4" />
            User
            {#if userDirtyCount > 0}
              <Badge variant="secondary" class="ml-1 h-5 min-w-5 px-1">
                {userDirtyCount}
              </Badge>
            {/if}
          </TabsTrigger>
          <TabsTrigger value="system" class="gap-2">
            <Monitor class="h-4 w-4" />
            System
            {#if systemDirtyCount > 0}
              <Badge variant="secondary" class="ml-1 h-5 min-w-5 px-1">
                {systemDirtyCount}
              </Badge>
            {/if}
          </TabsTrigger>
        </TabsList>

        <Button variant="outline" onclick={load} disabled={saving}>
          <RefreshCw class="mr-2 h-4 w-4" />
          Refresh
        </Button>
      </div>

      <TabsContent value="user">
        <EnvScopeCard
          title="User variables"
          description="Persistent for your account / user profile. Changes apply immediately without elevation."
          bind:search={userSearch}
          searchPlaceholder="Search user variables…"
          addDisabled={saving || permissions?.canEditUser === false}
          onadd={() => addRow('user')}
        >
          {@render variableList(userRows, userSearch)}
        </EnvScopeCard>
      </TabsContent>

      <TabsContent value="system">
        <EnvScopeCard
          title="System variables"
          description="Machine-wide variables visible to all users and services."
          bind:search={systemSearch}
          searchPlaceholder="Search system variables…"
          addDisabled={saving}
          onadd={() => addRow('system')}
        >
          {#if permissions && !permissions.canEditSystem}
            <Alert>
              <ShieldAlert class="h-4 w-4" />
              <AlertTitle>Administrator approval required</AlertTitle>
              <AlertDescription>
                Saving changes in this tab will open a UAC / administrator prompt. Portal stays open
                and picks up the new values when you approve.
              </AlertDescription>
            </Alert>
          {/if}
          {@render variableList(systemRows, systemSearch)}
        </EnvScopeCard>
      </TabsContent>
    </Tabs>

    <div class="flex flex-wrap gap-2">
      <Button onclick={save} disabled={saving || dirtyCount === 0}>
        {#if saving}
          <Loader2 class="mr-2 h-4 w-4 animate-spin" />
        {:else}
          <Save class="mr-2 h-4 w-4" />
        {/if}
        {needsElevation ? 'Save with elevation' : 'Save all changes'}
      </Button>
      <Button variant="outline" onclick={discardChanges} disabled={saving || dirtyCount === 0}>
        Discard
      </Button>
    </div>
  {/if}
</div>
