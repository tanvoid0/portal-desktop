<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { ShieldQuestion } from "@lucide/svelte";
  import type { PendingApproval } from "../types.js";

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
</script>

<div class="rounded-lg border border-amber-500/50 bg-amber-500/10 p-3">
  <div class="mb-2 flex items-center gap-2 text-sm font-medium text-amber-600 dark:text-amber-400">
    <ShieldQuestion class="h-4 w-4" />
    Approval required
  </div>

  <div class="mb-3 space-y-1 text-sm">
    <div>
      <span class="text-muted-foreground">tool</span>
      <span class="ml-2 font-mono text-xs">{pending.tool}</span>
    </div>
    <pre class="overflow-auto rounded bg-background p-2 text-xs"><code
        >{pending.summary}</code
      ></pre>
  </div>

  <div class="mb-3 flex items-center gap-2">
    <span class="text-xs text-muted-foreground">remember rule</span>
    <input
      bind:value={rule}
      class="flex-1 rounded border border-border bg-background px-2 py-1 font-mono text-xs"
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
