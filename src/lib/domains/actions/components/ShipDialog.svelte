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
  import { toast } from '$lib/utils/toast';
  import type { Project } from '$lib/domains/projects/types';
  import type { ReleaseSpec } from '../types';

  interface Props {
    project: Project;
    release: ReleaseSpec;
    open?: boolean;
  }
  let { project, release, open = $bindable(false) }: Props = $props();

  // `release.bump` is how a pipeline file says it reads $BUMP, and which is the default.
  let bump = $state('patch');
  let busy = $state(false);

  $effect(() => {
    if (open) bump = release.bump ?? 'patch';
  });

  async function ship() {
    busy = true;
    try {
      const run = await invoke<{ id: string }>('release_ship', {
        projectId: Number(project.id),
        variables: release.bump ? { BUMP: bump } : undefined,
      });
      open = false;
      await goto(`/projects/${project.id}/pipelines/run/${run.id}`);
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
      <AlertDialogTitle>Ship {project.name}?</AlertDialogTitle>
      <AlertDialogDescription>
        This runs <code>{release.ship}</code> for real. It can deploy servers and publish builds, and
        killing it midway does not undo what already went out.
      </AlertDialogDescription>
    </AlertDialogHeader>
    {#if release.bump}
      <div class="flex flex-wrap items-center gap-3 text-sm">
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
    <AlertDialogFooter>
      <AlertDialogCancel>Cancel</AlertDialogCancel>
      <AlertDialogAction disabled={busy} onclick={ship}>Ship</AlertDialogAction>
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
