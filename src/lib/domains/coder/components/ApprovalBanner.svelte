<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { ShieldQuestion } from "@lucide/svelte";
  import type { PendingApproval } from "../types.js";
  import { getToolCallDisplay } from "../utils/toolCallDisplay.js";

  interface Props {
    pending: PendingApproval;
    busy?: boolean;
    onDecision: (
      approve: boolean,
      remember: boolean,
      editedPattern?: string,
    ) => void;
  }

  let { pending, busy = false, onDecision }: Props = $props();

  // Editable allowlist pattern for the "accept & remember" action.
  // Initialized/reset from the current approval via the effect below.
  let rule = $state("");

  $effect(() => {
    rule = pending.suggested_rule;
  });

  const display = $derived(getToolCallDisplay(pending.tool, pending.arguments));
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

  <div class="mb-3 flex items-center gap-2">
    <span class="text-xs text-muted-foreground">remember rule</span>
    <Input
      bind:value={rule}
      class="flex-1 font-mono text-xs"
      placeholder="pattern to allow"
    />
  </div>

  <div class="flex flex-wrap gap-2">
    <Button size="sm" disabled={busy} onclick={() => onDecision(true, false)}>
      Accept once
    </Button>
    <Button
      size="sm"
      variant="secondary"
      disabled={busy}
      onclick={() => onDecision(true, true, rule)}
    >
      Accept &amp; remember
    </Button>
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
