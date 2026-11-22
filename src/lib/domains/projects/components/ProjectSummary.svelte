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

  // Keep the drive and the last folders visible; the full path stays in the tooltip.
  const shown = $derived.by(() => {
    const parts = project.path.split(/[\\/]/);
    const sep = project.path.includes('\\') ? '\\' : '/';
    return project.path.length > 56 && parts.length > 4
      ? `${parts[0]}${sep}…${sep}${parts.slice(-3).join(sep)}`
      : project.path;
  });

  async function copyPath() {
    await navigator.clipboard.writeText(project.path);
    copied = true;
    setTimeout(() => (copied = false), 1500);
  }
</script>

<div class="flex min-w-0 items-center gap-1 text-muted-foreground">
  <span class="truncate font-mono text-xs" title={project.path}>{shown}</span>
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
