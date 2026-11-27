<!--
  Release launcher, shared by Ship (ReleaseCard, ReleaseButton) and Rehearse
  (ReleaseCard). A scope picker chooses what runs:
    Full    — the release block's ship/rehearse workflow (apps + server).
    Server  — `release.server.ship` / `.rehearse` (workflow override); offered only
              when the release block declares one for this mode.
    Apps    — a subset: variables.APPS="portal_gym portal_task" (space-separated;
              the pocket_portal scripts already honour $APPS — no backend change).
  `apps` is sourced from the resolved ship plan (one node per app), never hardcoded.
-->
<script lang="ts">
  import { goto } from '$app/navigation';
  import { invoke } from '@tauri-apps/api/core';
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogDescription,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from '$lib/components/ui/alert-dialog';
  import { ToggleGroup, ToggleGroupItem } from '$lib/components/ui/toggle-group';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Label } from '$lib/components/ui/label';
  import { toast } from '$lib/utils/toast';
  import type { Project } from '$lib/domains/projects/types';
  import type { ReleaseSpec } from '../types';

  type Scope = 'full' | 'server' | 'apps';

  interface Props {
    project: Project;
    release: ReleaseSpec;
    /** Apps available for the subset scope, from the resolved ship plan. Empty hides it. */
    apps?: string[];
    /** `ship` (real release, local_only) or `rehearse` (dry run). */
    mode?: 'ship' | 'rehearse';
    open?: boolean;
  }
  let { project, release, apps = [], mode = 'ship', open = $bindable(false) }: Props = $props();

  const isShip = $derived(mode === 'ship');
  const serverWorkflow = $derived(isShip ? release.server?.ship : release.server?.rehearse);
  // `release.bump` is how a pipeline file says it reads $BUMP, and which is the default.
  let bump = $state('patch');
  let scope = $state<Scope>('full');
  let picked = $state<Set<string>>(new Set());
  let busy = $state(false);

  $effect(() => {
    if (open) {
      bump = release.bump ?? 'patch';
      scope = 'full';
      picked = new Set(apps);
    }
  });

  function toggle(app: string, on: boolean) {
    const next = new Set(picked);
    if (on) next.add(app);
    else next.delete(app);
    picked = next;
  }

  const canRun = $derived(scope !== 'apps' || picked.size > 0);

  async function run() {
    busy = true;
    try {
      const bumpVar: Record<string, string> =
        release.bump && scope !== 'server' ? { BUMP: bump } : {};
      let workflow: string | undefined;
      let variables: Record<string, string> | undefined;
      if (scope === 'server') {
        workflow = serverWorkflow;
      } else if (scope === 'apps') {
        variables = { APPS: apps.filter((a) => picked.has(a)).join(' '), ...bumpVar };
      } else if (Object.keys(bumpVar).length) {
        variables = bumpVar;
      }
      const started = await invoke<{ id: string }>(isShip ? 'release_ship' : 'release_rehearse', {
        projectId: Number(project.id),
        workflow,
        variables,
      });
      open = false;
      await goto(`/projects/${project.id}/pipelines/run/${started.id}`);
    } catch (e) {
      toast.error(String(e));
    } finally {
      busy = false;
    }
  }
</script>

<AlertDialog bind:open>
  <AlertDialogContent>
    <AlertDialogHeader>
      <AlertDialogTitle>{isShip ? 'Ship' : 'Rehearse'} {project.name}?</AlertDialogTitle>
      <AlertDialogDescription>
        {#if isShip}
          This runs for real. It can deploy servers and publish builds, and killing it midway does
          not undo what already went out.
        {:else}
          A dry run (<code>RELEASE_DRY_RUN=1</code>): gates and build, nothing deployed or
          published.
        {/if}
      </AlertDialogDescription>
    </AlertDialogHeader>

    <div class="space-y-3 text-sm">
      {#if serverWorkflow || apps.length}
        <div class="flex flex-wrap items-center gap-3">
          <span id="scope-label">Scope</span>
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            aria-labelledby="scope-label"
            bind:value={
              () => scope,
              (v) => {
                if (v) scope = v as Scope;
              }
            }
          >
            <ToggleGroupItem value="full">Full</ToggleGroupItem>
            {#if serverWorkflow}<ToggleGroupItem value="server">Server only</ToggleGroupItem>{/if}
            {#if apps.length}<ToggleGroupItem value="apps">Apps…</ToggleGroupItem>{/if}
          </ToggleGroup>
        </div>
      {/if}

      {#if scope === 'apps'}
        <div class="max-h-52 space-y-1.5 overflow-y-auto rounded-md border p-3">
          {#each apps as app (app)}
            <div class="flex items-center gap-2">
              <Checkbox
                id="app-{app}"
                checked={picked.has(app)}
                onCheckedChange={(v) => toggle(app, v === true)}
              />
              <Label for="app-{app}" class="font-normal">{app}</Label>
            </div>
          {/each}
        </div>
      {/if}

      {#if release.bump && scope !== 'server'}
        <div class="flex flex-wrap items-center gap-3">
          <span id="ship-bump-label">Version bump</span>
          <ToggleGroup
            type="single"
            variant="outline"
            size="sm"
            aria-labelledby="ship-bump-label"
            bind:value={
              () => bump,
              (v) => {
                if (v) bump = v;
              }
            }
          >
            <ToggleGroupItem value="patch">Patch</ToggleGroupItem>
            <ToggleGroupItem value="minor">Minor</ToggleGroupItem>
            <ToggleGroupItem value="major">Major</ToggleGroupItem>
          </ToggleGroup>
        </div>
      {/if}
    </div>

    <AlertDialogFooter>
      <AlertDialogCancel>Cancel</AlertDialogCancel>
      <AlertDialogAction disabled={busy || !canRun} onclick={run}>
        {isShip ? 'Ship' : 'Rehearse'}
      </AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
