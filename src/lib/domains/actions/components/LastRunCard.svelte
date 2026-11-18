<!-- Fallback result card for a run that has no persisted execution record to show. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';
  import type { ActionRunResult } from '$lib/domains/actions';

  interface Props {
    result: ActionRunResult;
    onOpenExecution: (executionId: string) => void;
  }

  let { result, onOpenExecution }: Props = $props();
</script>

<SectionCard
  title="Last run"
  description="{result.success ? 'Succeeded' : 'Failed'} · {result.cwd}"
  titleClass="text-base"
  contentClass="space-y-3"
>
  {#each result.steps as step (step.actionId + step.name)}
    <div class="space-y-1 rounded border p-2">
      <div class="flex items-center justify-between gap-2 text-sm">
        <span class="font-medium">{step.name}</span>
        <Badge
          variant={step.status === 'success'
            ? 'default'
            : step.status === 'skipped' || step.status === 'cancelled'
              ? 'secondary'
              : 'destructive'}
        >
          {step.status}
        </Badge>
      </div>
      {#if step.command}
        <code class="block truncate text-xs text-muted-foreground">
          {step.command}
        </code>
      {/if}
      {#if step.exitCode != null}
        <p class="text-xs text-muted-foreground">
          Exit code {step.exitCode}
        </p>
      {/if}
      {#if step.error}
        <pre
          class="max-h-32 overflow-auto text-xs whitespace-pre-wrap text-destructive">{step.error}</pre>
      {/if}
      {#if step.output?.trim()}
        <pre
          class="max-h-40 overflow-auto rounded bg-muted px-2 py-1 text-xs whitespace-pre-wrap">{step.output}</pre>
      {/if}
      {#if step.executionId}
        <Button
          type="button"
          size="sm"
          variant="outline"
          class="mt-1"
          onclick={() => onOpenExecution(step.executionId!)}
        >
          View full details
        </Button>
      {/if}
    </div>
  {/each}
</SectionCard>
