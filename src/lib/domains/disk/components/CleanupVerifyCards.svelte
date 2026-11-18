<!-- AI verification: live progress while running, advisory result once done. -->
<script lang="ts">
  import type { VerificationResult, VerifyProgress } from '../types';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';

  let {
    verifying,
    progress,
    verification,
    error,
    oncancel,
    ondismiss,
    ondeselectFlagged,
  }: {
    verifying: boolean;
    progress: VerifyProgress | null;
    verification: VerificationResult | null;
    error: string;
    oncancel: () => void;
    ondismiss: () => void;
    ondeselectFlagged: () => void;
  } = $props();

  const counts = $derived.by(() => {
    const c = { safe: 0, review: 0, dangerous: 0 };
    verification?.verdicts.forEach((v) => {
      // `verdict` is a plain String on the Rust side — the synthesizer's output
      // is model text, so the parser stays tolerant and normalizes anything it
      // does not recognise to "review". The type cannot say that, so do it here.
      if (v.verdict in c) c[v.verdict as keyof typeof c]++;
      else c.review++;
    });
    return c;
  });
</script>

{#snippet header()}
  <CardHeader class="divider-edge-b divider-edge-full bg-muted/40 px-4 py-2.5 [.border-b]:pb-2.5">
    <div class="flex items-center gap-2">
      <CardTitle class="text-sm font-medium">AI verification</CardTitle>
      {#if progress && verifying}
        <span class="text-xs text-muted-foreground"
          >process #{progress.processId} · {progress.status.replace(/_/g, ' ')}</span
        >
        <Button variant="outline" size="sm" onclick={oncancel} class="ml-auto">Stop</Button>
      {:else}
        {#if verification}
          <span class="text-xs text-muted-foreground"
            >process #{verification.processId} · {verification.status}</span
          >
        {/if}
        {#if verification?.gated}
          <Badge
            variant="outline"
            class="border-status-warning/30 bg-status-warning-bg text-status-warning"
            >paused at a review gate</Badge
          >
        {/if}
        <Button variant="outline" size="sm" onclick={ondismiss} class="ml-auto">Dismiss</Button>
      {/if}
    </div>
  </CardHeader>
{/snippet}

{#if verifying && progress}
  <Card class="mb-4 gap-0 overflow-hidden py-0">
    {@render header()}
    <CardContent class="px-4 py-3">
      {#if progress.tasks.length === 0}
        <div class="flex items-center gap-2 text-sm text-muted-foreground">
          <span class="h-1.5 w-1.5 animate-pulse rounded-full bg-primary"></span>Planning the
          review…
        </div>
      {:else}
        <ul class="space-y-1.5">
          {#each progress.tasks as t, i (i)}
            <li class="flex items-center gap-2 text-sm text-foreground">
              <span
                class="h-1.5 w-1.5 rounded-full {t.status === 'completed'
                  ? 'bg-status-success'
                  : t.status === 'failed'
                    ? 'bg-status-error'
                    : t.status.includes('progress') || t.status === 'running'
                      ? 'animate-pulse bg-primary'
                      : 'bg-muted-foreground'}"
              ></span>
              <span>{t.role}</span>
              <span class="text-xs text-muted-foreground">{t.status.replace(/_/g, ' ')}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </CardContent>
  </Card>
{/if}

{#if error || verification}
  <Card class="mb-4 gap-0 overflow-hidden py-0">
    {@render header()}
    <CardContent class="px-4 py-3">
      <p class="mb-3 text-xs text-muted-foreground">
        Advisory only — the agents review deletion candidates but never delete or change risk. You
        still tick every item.
      </p>
      {#if verification && verification.verdicts.length > 0}
        <div class="mb-3 flex flex-wrap items-center gap-3">
          <span class="text-sm text-foreground">
            <span class="text-status-success">{counts.safe} safe</span> ·
            <span class="text-status-warning">{counts.review} review</span>
            · <span class="text-status-error">{counts.dangerous} dangerous</span>
          </span>
          {#if counts.review + counts.dangerous > 0}
            <Button variant="outline" size="sm" onclick={ondeselectFlagged}
              >Deselect {counts.review + counts.dangerous} flagged</Button
            >
          {/if}
        </div>
      {/if}
      {#if error}
        <Alert variant="destructive" class="mb-3">
          <AlertDescription class="whitespace-pre-wrap">{error}</AlertDescription>
        </Alert>
      {/if}
      {#if verification}
        {#if verification.notes.length === 0}
          <div class="text-sm text-muted-foreground">
            No agent notes returned{verification.gated ? ' before the gate.' : '.'}
          </div>
        {:else}
          <div class="space-y-3">
            {#each verification.notes as n (n.taskId)}
              <Card class="gap-0 py-3">
                <CardContent class="px-3">
                  <div class="mb-1.5 flex items-center gap-2">
                    <span class="text-xs font-medium text-foreground">{n.role}</span>
                    <span class="text-xs text-muted-foreground">#{n.taskId} · {n.status}</span>
                  </div>
                  <div class="text-sm break-words whitespace-pre-wrap text-muted-foreground">
                    {n.output}
                  </div>
                </CardContent>
              </Card>
            {/each}
          </div>
        {/if}
      {/if}
    </CardContent>
  </Card>
{/if}
