<!-- Resolved commands for the queued blocks. -->
<script lang="ts">
  import { PageLoading } from '$lib/components/shell';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';
  import type { ResolvedExecutionStep } from '$lib/domains/automation';

  interface Props {
    cwd: string;
    loading: boolean;
    steps: ResolvedExecutionStep[];
  }

  let { cwd, loading, steps }: Props = $props();
</script>

<SectionCard title="Preview" description="Resolved commands for {cwd || '…'}">
  {#if loading}
    <PageLoading message="Resolving commands…" />
  {:else}
    <div class="space-y-2">
      {#each steps as step, i}
        <div class="rounded-md border bg-muted/20 p-3">
          <p class="mb-1 text-sm font-medium">
            {i + 1}. {step.name}
          </p>
          <code class="block text-xs break-all text-muted-foreground">
            {step.command}
          </code>
        </div>
      {/each}
    </div>
  {/if}
</SectionCard>
