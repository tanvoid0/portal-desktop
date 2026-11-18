<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { Card, CardContent } from '$lib/components/ui/card';
  import type { DiscoveryChange } from '$lib/domains/projects/generated/DiscoveryChange';

  interface Props {
    changes: DiscoveryChange[];
  }
  let { changes }: Props = $props();
</script>

<Card>
  <CardContent class="space-y-1 p-3">
    {#each changes as c (c.at + c.kind + c.path)}
      <div class="flex flex-wrap items-center gap-2 text-sm">
        <Badge
          variant={c.kind === 'removed'
            ? 'destructive'
            : c.kind === 'added'
              ? 'default'
              : 'secondary'}>{c.kind}</Badge
        >
        <span class="flex-1 truncate font-mono text-xs">{c.path}</span>
        {#if c.from && c.from !== c.path}
          <span class="truncate font-mono text-xs text-muted-foreground">was {c.from}</span>
        {:else if c.kind === 'moved' && c.detail}
          <!-- Same id, new folder: the old absolute path is the whole story. -->
          <span class="truncate font-mono text-xs text-muted-foreground">was {c.detail}</span>
        {/if}
        <span class="text-xs text-muted-foreground">{new Date(c.at).toLocaleString()}</span>
      </div>
    {:else}
      <p class="py-8 text-center text-sm text-muted-foreground">
        Nothing yet — the first scan is the baseline. Rescan later and what appeared, moved or
        vanished lands here.
      </p>
    {/each}
  </CardContent>
</Card>
