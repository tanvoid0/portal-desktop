<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Card, CardContent } from '$lib/components/ui/card';
  import ChevronRight from '~icons/fa6-solid/chevron-right';
  import ChevronDown from '~icons/fa6-solid/chevron-down';
  import type { DiscoveredProject } from '$lib/domains/projects/generated/DiscoveredProject';
  import { ageLabel } from '$lib/domains/projects/utils/auditFormat';
  import AuditStatusBadge from './AuditStatusBadge.svelte';

  interface Props {
    projects: DiscoveredProject[];
    searching: boolean;
    onInspect: (p: DiscoveredProject) => void;
  }
  let { projects, searching, onInspect }: Props = $props();

  let expanded = $state<Record<string, boolean>>({});

  /** Groups projects by their bucket, preserving activity order. */
  const tree = $derived.by(() => {
    const byArea = new Map<string, Map<string, DiscoveredProject[]>>();
    for (const p of projects) {
      if (!byArea.has(p.area)) byArea.set(p.area, new Map());
      const buckets = byArea.get(p.area)!;
      const bucket = p.bucket || p.area;
      if (!buckets.has(bucket)) buckets.set(bucket, []);
      buckets.get(bucket)!.push(p);
    }
    return byArea;
  });
</script>

<Card>
  <CardContent class="p-3 font-mono text-sm">
    {#each [...tree.entries()] as [area, buckets] (area)}
      <div class="py-1">
        <Button
          variant="ghost"
          class="h-auto justify-start gap-1 rounded px-1 py-0 font-mono text-sm hover:bg-muted"
          onclick={() => (expanded = { ...expanded, [area]: !expanded[area] })}
        >
          {#if expanded[area]}<ChevronDown class="h-3 w-3" />{:else}<ChevronRight
              class="h-3 w-3"
            />{/if}
          <span class="font-semibold text-primary">{area}</span>
          <span class="text-xs text-muted-foreground">
            {[...buckets.values()].reduce((a, b) => a + b.length, 0)} projects
          </span>
        </Button>
        {#if expanded[area]}
          <div class="ml-4 border-l border-muted pl-3">
            {#each [...buckets.entries()] as [bucket, items] (bucket)}
              <div class="py-1">
                <div class="text-xs text-muted-foreground">{bucket}</div>
                {#each items as p (p.path)}
                  <Button
                    variant="ghost"
                    class="h-auto w-full justify-start gap-2 rounded px-1 py-0 text-left font-mono text-sm hover:bg-muted"
                    onclick={() => onInspect(p)}
                  >
                    <span class="flex-1 truncate">{p.name}</span>
                    {#if p.moduleCount > 1}
                      <Badge variant="outline">{p.moduleCount} modules</Badge>
                    {/if}
                    <AuditStatusBadge status={p.status} />
                    <span class="text-xs text-muted-foreground">{ageLabel(p.ageDays)}</span>
                  </Button>
                {/each}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {:else}
      <p class="py-8 text-center font-sans text-sm text-muted-foreground">
        {searching ? 'Nothing matches that search.' : 'Nothing to group yet — run a scan first.'}
      </p>
    {/each}
  </CardContent>
</Card>
