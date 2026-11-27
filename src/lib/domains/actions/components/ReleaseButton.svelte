<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { Button } from '$lib/components/ui/button';
  import { toast } from '$lib/utils/toast';
  import Rocket from '~icons/fa6-solid/rocket';
  import type { Project } from '$lib/domains/projects/types';
  import type { ReleaseDescription, ReleaseSpec } from '../types';
  import ShipDialog from './ShipDialog.svelte';

  /** Header shortcut to Ship. Renders nothing until the project declares a `release:`. */
  let { project }: { project: Project } = $props();

  let found = $state<{ projectId: string; release: ReleaseSpec } | null>(null);
  let open = $state(false);
  // A release declared on the project you just left must not show on this one.
  const release = $derived(found?.projectId === project.id ? found.release : null);

  async function describe() {
    const projectId = project.id;
    const described = await invoke<ReleaseDescription | null>('release_describe', {
      projectId: Number(projectId),
    });
    // A late reply for the project you just left must not overwrite this one's.
    if (projectId !== project.id) return null;
    found = described ? { projectId, release: described.release } : null;
    return described;
  }

  // ponytail: read once per project; a release file created in the Release tab shows
  // here on the next visit.
  $effect(() => {
    describe().catch(() => (found = null));
  });

  // Re-read on click: the pipeline file may have changed since the page loaded.
  async function confirmShip() {
    try {
      if (await describe()) open = true;
    } catch (e) {
      toast.error(String(e));
    }
  }
</script>

{#if release}
  <Button variant="outline" class="gap-2" onclick={confirmShip}>
    <Rocket class="h-4 w-4" />
    Release…
  </Button>
  <ShipDialog {project} {release} bind:open />
{/if}
