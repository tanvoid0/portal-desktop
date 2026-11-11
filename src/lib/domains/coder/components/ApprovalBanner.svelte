<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import ShieldQuestion from '~icons/fa6-solid/shield-halved';
  import type { PendingApproval } from '../types.js';
  import { getToolCallDisplay } from '../utils/toolCallDisplay.js';

  interface Props {
    pending: PendingApproval;
    busy?: boolean;
    /** Hide "Accept & remember" + the rule input for callers with no remember-rule
     *  persistence (e.g. the agents domain — see QA-112). Defaults on for coder. */
    showRemember?: boolean;
    onDecision: (approve: boolean, remember: boolean, editedPattern?: string) => void;
  }

  let { pending, busy = false, showRemember = true, onDecision }: Props = $props();

  // Editable allowlist pattern for the "accept & remember" action.
  // Initialized/reset from the current approval via the effect below.
  let rule = $state('');

  $effect(() => {
    rule = pending.suggested_rule;
  });

  // `arguments` is `serde_json::Value` on the Rust side, so it arrives as
  // `unknown`. Tool arguments are always a JSON object in practice; anything
  // else renders as no arguments rather than throwing.
  const display = $derived(
    getToolCallDisplay(
      pending.tool,
      (typeof pending.arguments === 'object' && pending.arguments !== null
        ? pending.arguments
        : {}) as Record<string, unknown>
    )
  );
</script>

<div class="rounded-lg border border-amber-500/50 bg-amber-500/10 p-3">
  <div class="mb-2 flex items-center gap-2 text-sm font-medium text-amber-600 dark:text-amber-400">
    <ShieldQuestion class="h-4 w-4" />
    Approval required
  </div>

  <div class="mb-3 space-y-1 text-sm">
    <div class="font-medium">{display.label}</div>
    {#if display.detail}
      <div class="font-mono text-xs text-muted-foreground">{display.detail}</div>
    {/if}
  </div>

  {#if showRemember}
    <div class="mb-3 flex items-center gap-2">
      <span class="text-xs text-muted-foreground">remember rule</span>
      <Input bind:value={rule} class="flex-1 font-mono text-xs" placeholder="pattern to allow" />
    </div>
  {/if}

  <div class="flex flex-wrap gap-2">
    <Button size="sm" disabled={busy} onclick={() => onDecision(true, false)}>Accept once</Button>
    {#if showRemember}
      <Button
        size="sm"
        variant="secondary"
        disabled={busy}
        onclick={() => onDecision(true, true, rule)}
      >
        Accept &amp; remember
      </Button>
    {/if}
    <Button
      size="sm"
      variant="destructive"
      disabled={busy}
      onclick={() => onDecision(false, false)}
    >
      Reject
    </Button>
  </div>
</div>
