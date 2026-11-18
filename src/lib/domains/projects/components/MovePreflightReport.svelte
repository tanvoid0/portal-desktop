<!-- Preflight result: size summary, nested projects, blockers, port warnings and excluded dirs. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Label } from '$lib/components/ui/label';
  import TriangleExclamation from '~icons/fa6-solid/triangle-exclamation';
  import CircleXmark from '~icons/fa6-solid/circle-xmark';
  import Loader2 from '~icons/fa6-solid/spinner';
  import X from '~icons/fa6-solid/xmark';
  import { fmtBytes } from '$lib/domains/disk/utils';
  import type { PreflightResult, PortWarning } from '../api/projectMoveApi';

  interface Props {
    preflight: PreflightResult;
    includeRegenerable: boolean;
    acknowledged: Set<number>;
    endErrors: Record<number, string>;
    endingIndex: number | null;
    onEndProcess: (index: number, w: PortWarning) => void;
  }

  let {
    preflight,
    includeRegenerable,
    acknowledged = $bindable(),
    endErrors,
    endingIndex,
    onEndProcess,
  }: Props = $props();
</script>

<p class="text-xs text-muted-foreground tabular-nums">
  {preflight.file_count} files · {fmtBytes(preflight.total_bytes)}
  {#if preflight.available_bytes > 0}
    · {fmtBytes(preflight.available_bytes)} free at destination
  {/if}
</p>

{#if preflight.nested_projects.length > 0}
  <p class="text-xs text-muted-foreground">
    Also registered under this path, and moving with it: {preflight.nested_projects
      .map((p) => p.name)
      .join(', ')}
  </p>
{/if}

{#if preflight.blockers.length > 0}
  <div class="space-y-1.5">
    {#each preflight.blockers as b, i (i)}
      <div
        class="flex items-start gap-2 rounded-md border border-destructive/50 bg-destructive/10 px-3 py-2 text-sm text-destructive"
      >
        <CircleXmark class="mt-0.5 h-4 w-4 shrink-0" />
        {b.detail}
      </div>
    {/each}
  </div>
{/if}

{#if preflight.warnings.length > 0}
  <div class="space-y-1.5">
    {#each preflight.warnings as w, i (i)}
      <div
        class="flex items-start gap-2 rounded-md border border-amber-300 bg-amber-50 px-3 py-2 text-sm dark:border-amber-800 dark:bg-amber-950"
      >
        <TriangleExclamation class="mt-0.5 h-4 w-4 shrink-0 text-amber-600 dark:text-amber-400" />
        <div class="min-w-0 flex-1 space-y-1">
          <p class="text-amber-800 dark:text-amber-200">
            Port {w.port}: {w.process_name} (pid {w.pid}) — {w.reason}
          </p>
          {#if endErrors[i]}
            <p class="text-xs text-destructive">{endErrors[i]}</p>
          {/if}
          <div class="flex items-center gap-2">
            <Label class="flex items-center gap-1.5 text-xs text-amber-800 dark:text-amber-200">
              <Checkbox
                checked={acknowledged.has(i)}
                onCheckedChange={(v) => {
                  const next = new Set(acknowledged);
                  if (v === true) next.add(i);
                  else next.delete(i);
                  acknowledged = next;
                }}
              />
              Proceed anyway
            </Label>
            <Button
              size="sm"
              variant="outline"
              class="h-7 gap-1 text-xs"
              disabled={!w.killable || endingIndex === i}
              title={!w.killable ? w.reason : undefined}
              onclick={() => onEndProcess(i, w)}
            >
              {#if endingIndex === i}
                <Loader2 class="h-3 w-3 animate-spin" />
              {:else}
                <X class="h-3 w-3" />
              {/if}
              End process
            </Button>
          </div>
        </div>
      </div>
    {/each}
  </div>
{/if}

{#if !includeRegenerable && preflight.skipped_dirs.length > 0}
  <div class="space-y-1 rounded-md border border-border px-3 py-2 text-xs text-muted-foreground">
    <p class="font-medium">Excluded (regenerable):</p>
    {#each preflight.skipped_dirs as dir (dir.rel)}
      <p class="flex justify-between gap-2 tabular-nums">
        <span class="truncate font-mono">{dir.rel}</span>
        <span class="shrink-0">{fmtBytes(dir.bytes)}</span>
      </p>
    {/each}
  </div>
{/if}
