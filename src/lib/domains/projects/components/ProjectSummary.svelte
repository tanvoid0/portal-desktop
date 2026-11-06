<!--
  Path strip for the project detail page: the folder, with copy.
-->

<script lang="ts">
  import Check from '~icons/fa6-solid/check';
  import Copy from '~icons/fa6-solid/copy';
  import { Button } from '$lib/components/ui/button';
  import type { Project } from '$lib/domains/projects/types';

  interface Props {
    project: Project;
  }

  let { project }: Props = $props();

  let copied = $state(false);

  async function copyPath() {
    await navigator.clipboard.writeText(project.path);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<div class="flex min-w-0 items-center gap-1 text-muted-foreground">
  <span class="truncate font-mono text-xs" title={project.path}>{project.path}</span>
  <Button
    variant="ghost"
    size="sm"
    class="h-6 w-6 shrink-0 p-0"
    onclick={copyPath}
    title="Copy path"
  >
    {#if copied}
      <Check class="h-3.5 w-3.5" />
    {:else}
      <Copy class="h-3.5 w-3.5" />
    {/if}
  </Button>
</div>
