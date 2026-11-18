<!-- "Run failed" card: the failure kind/hint and the Diagnose & fix action with its live status. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import Stethoscope from '~icons/fa6-solid/stethoscope';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';
  import type { ActionFailure } from '../types';
  import type { DoctorRun } from '../state/doctor.svelte';
  import AgentRunFollowUp from './AgentRunFollowUp.svelte';

  interface Props {
    failure: ActionFailure;
    run: DoctorRun;
    /** Doctor or generator is already working, so Diagnose is blocked. */
    busy: boolean;
    onDiagnose: () => void;
    onOpenThread: (id: string) => void;
  }

  let { failure: f, run, busy, onDiagnose, onOpenThread }: Props = $props();

  const statusLabel = $derived(
    run.status === 'diagnosing'
      ? 'Diagnosing…'
      : run.status === 'verifying'
        ? 'Verifying…'
        : run.status === 'fixed'
          ? 'Fixed'
          : run.status === 'awaiting-verify'
            ? 'Fix proposed'
            : run.unavailable
              ? 'Could not start'
              : 'Gave up'
  );
</script>

<SectionCard
  title="Run failed"
  description={f.hint}
  titleClass="text-base text-destructive"
  headerClass="pb-2"
  class="border-destructive/40"
  contentClass="flex flex-wrap items-center gap-2"
>
  {#snippet actions()}
    <Badge variant="outline" class="capitalize">{f.kind}</Badge>
  {/snippet}
  <Button
    type="button"
    size="sm"
    variant="outline"
    class="gap-1"
    disabled={busy || run.status === 'awaiting-verify'}
    onclick={onDiagnose}
  >
    <Stethoscope class="h-3.5 w-3.5" />
    Diagnose & fix
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
    busy={run.status === 'diagnosing'}
    {onOpenThread}
  />
</SectionCard>
