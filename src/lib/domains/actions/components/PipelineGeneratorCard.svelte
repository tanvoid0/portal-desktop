<!-- "Generate a pipeline" — describe a requirement, an agent writes .portal/pipeline.yml. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Textarea } from '$lib/components/ui/textarea';
  import Sparkles from '~icons/fa6-solid/wand-magic-sparkles';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';
  import type { GenerateRun } from '../state/doctor.svelte';
  import AgentRunFollowUp from './AgentRunFollowUp.svelte';

  interface Props {
    requirement: string;
    run: GenerateRun;
    running: boolean;
    /** Another run (action or doctor) is busy, so Generate is blocked too. */
    blocked: boolean;
    onGenerate: () => void;
    onOpenThread: (id: string) => void;
  }

  let {
    requirement = $bindable(),
    run,
    running,
    blocked,
    onGenerate,
    onOpenThread,
  }: Props = $props();

  const statusLabel = $derived(
    run.status === 'generating'
      ? 'Generating…'
      : run.status === 'validating'
        ? 'Validating…'
        : run.status === 'previewing'
          ? 'Dry-checking…'
          : run.status === 'done'
            ? 'Done'
            : 'Failed'
  );
</script>

<SectionCard
  title="Generate a pipeline"
  icon={Sparkles}
  titleClass="text-base"
  headerClass="pb-2"
  contentClass="space-y-2"
>
  {#snippet description()}
    Describe what you want and an agent writes or extends
    <code class="text-xs">.portal/pipeline.yml</code> for this project's stack. Nothing is committed —
    review the changes in Coder.
  {/snippet}
  <Textarea
    bind:value={requirement}
    rows={3}
    placeholder="e.g. Add a lint + typecheck gate and a release workflow that builds then tests."
    disabled={running}
  />
  <div class="flex flex-wrap items-center gap-2">
    <Button
      type="button"
      size="sm"
      class="gap-1"
      disabled={running || blocked || !requirement.trim()}
      onclick={onGenerate}
    >
      <Sparkles class="h-3.5 w-3.5" />
      {running ? 'Generating…' : 'Generate'}
    </Button>
    {#if run.status !== 'idle'}
      <span class="text-xs text-muted-foreground">
        {statusLabel}
        {#if run.message}· {run.message}{/if}
      </span>
    {/if}
    <AgentRunFollowUp
      waiting={run.waiting}
      unavailable={run.unavailable}
      threadId={run.threadId}
      busy={running}
      {onOpenThread}
    />
  </div>
</SectionCard>
