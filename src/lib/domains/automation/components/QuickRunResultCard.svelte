<!-- Per-step outcome of a finished automation run. -->
<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import CheckCircle from '~icons/fa6-solid/circle-check';
  import XCircle from '~icons/fa6-solid/circle-xmark';
  import Loader2 from '~icons/fa6-solid/spinner';
  import SkipForward from '~icons/fa6-solid/forward-step';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';
  import type { AutomationRunResult } from '$lib/domains/automation';

  interface Props {
    result: AutomationRunResult;
  }

  let { result }: Props = $props();

  function stepStatusIcon(status: string) {
    switch (status) {
      case 'success':
        return CheckCircle;
      case 'failed':
        return XCircle;
      case 'skipped':
        return SkipForward;
      default:
        return Loader2;
    }
  }
</script>

<SectionCard
  title={result.success ? 'Run completed' : 'Run finished with errors'}
  description={result.cwd}
  icon={result.success ? CheckCircle : XCircle}
  contentClass="space-y-3"
>
  {#each result.steps as step, i}
    {@const Icon = stepStatusIcon(step.status)}
    <div class="rounded-md border p-3">
      <div class="mb-2 flex items-center gap-2">
        <Icon
          class="h-4 w-4 {step.status === 'success'
            ? 'text-green-600'
            : step.status === 'failed'
              ? 'text-destructive'
              : 'text-muted-foreground'}"
        />
        <span class="text-sm font-medium">
          {i + 1}. {step.name}
        </span>
        <Badge variant="outline" class="text-xs capitalize">
          {step.status}
        </Badge>
        {#if step.exitCode != null}
          <span class="text-xs text-muted-foreground">
            exit {step.exitCode}
          </span>
        {/if}
      </div>
      <code class="mb-2 block text-xs break-all text-muted-foreground">
        {step.command}
      </code>
      {#if step.output}
        <pre
          class="max-h-40 overflow-auto rounded bg-muted p-2 text-xs whitespace-pre-wrap">{step.output}</pre>
      {/if}
      {#if step.error}
        <p class="mt-1 text-xs text-destructive">{step.error}</p>
      {/if}
    </div>
  {/each}
</SectionCard>
