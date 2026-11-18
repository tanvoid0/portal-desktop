<!--
	One compact row under the top bar for background work (scans, runs, downloads):
	title, progress, stop. Several jobs collapse to "+N"; click to expand the rest.
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import XMark from '~icons/fa6-solid/xmark';
  import ChevronDown from '~icons/fa6-solid/chevron-down';
  import { Button } from '$lib/components/ui/button';
  import { Progress } from '$lib/components/ui/progress';
  import { backgroundJobs } from '$lib/domains/shared/jobs/backgroundJobs.svelte';

  onMount(() => backgroundJobs.start());

  let open = $state(false);
  const jobs = $derived(backgroundJobs.items);
  const shown = $derived(open ? jobs : jobs.slice(0, 1));
</script>

{#if jobs.length > 0}
  <div
    class="max-h-24 shrink-0 divide-y overflow-y-auto border-b bg-muted/40 px-3"
    role="status"
    aria-label="Background tasks"
  >
    {#each shown as job, i (job.id)}
      <div class="flex h-7 min-w-0 items-center gap-2 text-xs">
        <Button
          variant="link"
          class="h-auto max-w-[12rem] shrink-0 justify-start truncate p-0 text-xs font-medium"
          disabled={!job.href}
          onclick={() => job.href && goto(job.href)}
        >
          {job.title}
        </Button>
        <Progress
          value={job.percent ?? 100}
          class="h-1 max-w-64 min-w-12 flex-1 {job.percent == null ? 'animate-pulse' : ''}"
        />
        <span class="hidden min-w-0 flex-1 truncate text-muted-foreground sm:inline">
          {job.percent != null ? `${Math.round(job.percent)}% · ` : ''}{job.detail ?? ''}
        </span>
        {#if i === 0 && jobs.length > 1}
          <Button
            variant="ghost"
            size="sm"
            class="h-6 shrink-0 gap-1 px-1.5 text-xs"
            aria-expanded={open}
            onclick={() => (open = !open)}
          >
            +{jobs.length - 1}
            <ChevronDown class="size-2.5 transition-transform {open ? 'rotate-180' : ''}" />
          </Button>
        {/if}
        {#if job.cancel}
          <Button
            variant="ghost"
            size="icon"
            class="size-6 shrink-0"
            aria-label="Stop {job.title}"
            onclick={() => job.cancel?.()}
          >
            <XMark class="size-3" />
          </Button>
        {/if}
      </div>
    {/each}
  </div>
{/if}
