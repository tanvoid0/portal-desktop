<!-- The agent's proposed command change, shown for review before anything runs (QA-153). -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import Play from '~icons/fa6-solid/play';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';
  import type { VerifyProposal } from '../state/doctor.svelte';
  import LabeledCommand from './LabeledCommand.svelte';

  interface Props {
    proposal: VerifyProposal;
    reviewThreadId?: string;
    onVerify: () => void;
    onOpenThread: (id: string) => void;
  }

  let { proposal: pr, reviewThreadId, onVerify, onOpenThread }: Props = $props();
</script>

<SectionCard
  title="Review the agent's change"
  description="Nothing has run. {pr.note ?? 'The agent changed what this action runs.'}"
  titleClass="text-base"
  headerClass="pb-2"
  class="border-amber-500/40"
  contentClass="space-y-2"
>
  <LabeledCommand label="Was" command={pr.oldCommand} />
  <LabeledCommand label="Agent wrote (not run yet)" command={pr.newCommand} />
  {#if pr.scriptChanges.length}
    <div>
      <p class="mb-1 text-xs font-medium text-muted-foreground">package.json scripts changed</p>
      {#each pr.scriptChanges as line (line)}
        <code class="mb-1 block rounded bg-muted px-2 py-1 text-xs break-all">{line}</code>
      {/each}
    </div>
  {/if}
  <div class="flex flex-wrap items-center gap-2">
    <Button type="button" size="sm" class="gap-1" onclick={onVerify}>
      <Play class="h-3.5 w-3.5" />
      Verify
    </Button>
    {#if reviewThreadId}
      <Button type="button" size="sm" variant="link" onclick={() => onOpenThread(reviewThreadId)}>
        Review changes in Coder
      </Button>
    {/if}
  </div>
</SectionCard>
