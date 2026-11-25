<script lang="ts">
  import { createMutation, useQueryClient } from '@tanstack/svelte-query';
  import { open } from '@tauri-apps/plugin-dialog';
  import { Button } from '$lib/components/ui/button';
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from '$lib/components/ui/card';
  import { toast } from '$lib/utils/toast';
  import { getErrorMessage } from '$lib/utils/errorMessage';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { ignoreProject, resolveProjectPath } from '../api/syncApi';
  import { createUnresolvedQuery } from '../queries/syncQueries';

  const client = useQueryClient();
  const query = createUnresolvedQuery();
  const rows = $derived(query.data ?? []);

  const refresh = () =>
    Promise.all([
      client.invalidateQueries({ queryKey: queryKeys.sync.all }),
      client.invalidateQueries({ queryKey: queryKeys.projects.all }),
    ]);
  const onError = (e: unknown) => toast.error(getErrorMessage(e, 'Failed'));

  const resolve = createMutation(() => ({
    mutationFn: async (projectId: number) => {
      const dir = await open({ directory: true, multiple: false });
      if (typeof dir === 'string') await resolveProjectPath(projectId, dir);
    },
    onSuccess: refresh,
    onError,
  }));
  const ignore = createMutation(() => ({
    mutationFn: (projectId: number) => ignoreProject(projectId),
    onSuccess: refresh,
    onError,
  }));
  const ignoreAll = createMutation(() => ({
    mutationFn: async () => {
      for (const r of rows) await ignoreProject(r.projectId);
    },
    onSuccess: refresh,
    onError,
  }));
</script>

{#if rows.length > 0}
  <Card>
    <CardHeader>
      <div class="flex flex-wrap items-center justify-between gap-2">
        <CardTitle>Projects without a folder</CardTitle>
        <Button
          size="sm"
          variant="outline"
          disabled={ignoreAll.isPending}
          onclick={() => ignoreAll.mutate()}
        >
          Ignore all
        </Button>
      </div>
      <CardDescription>
        {rows.length}
        {rows.length === 1 ? 'project has' : 'projects have'} no folder on this machine yet. Locate the
        folder here, or Ignore the project if you do not use it on this machine.
      </CardDescription>
    </CardHeader>
    <CardContent class="space-y-2">
      {#each rows as p (p.projectId)}
        <div class="flex flex-wrap items-center justify-between gap-2 rounded-md border p-3">
          <div class="min-w-0">
            <div class="truncate font-medium">{p.name}</div>
            {#if p.lastKnownPath}
              <div class="truncate text-xs text-muted-foreground" title={p.lastKnownPath}>
                {p.lastKnownPath} - from another machine
              </div>
            {/if}
          </div>
          <div class="flex gap-2">
            <Button
              size="sm"
              disabled={resolve.isPending}
              onclick={() => resolve.mutate(p.projectId)}
            >
              Locate folder...
            </Button>
            <Button
              size="sm"
              variant="ghost"
              disabled={ignore.isPending}
              onclick={() => ignore.mutate(p.projectId)}
            >
              Ignore
            </Button>
          </div>
        </div>
      {/each}
    </CardContent>
  </Card>
{/if}
