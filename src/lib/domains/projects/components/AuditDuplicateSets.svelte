<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import Copy from '~icons/fa6-solid/copy';
  import type { DiscoveredProject } from '$lib/domains/projects/generated/DiscoveredProject';
  import type { DiscoveryReport } from '$lib/domains/projects/generated/DiscoveryReport';
  import { ageLabel } from '$lib/domains/projects/utils/auditFormat';

  interface Props {
    sets: { cluster: DiscoveryReport['clusters'][number]; members: DiscoveredProject[] }[];
    onInspect: (p: DiscoveredProject) => void;
  }
  let { sets, onInspect }: Props = $props();
</script>

<div class="space-y-3">
  {#each sets as { cluster, members } (cluster.kind + cluster.key)}
    <Card>
      <CardHeader class="pb-2">
        <CardTitle class="flex items-center gap-2 text-sm">
          <Copy class="h-4 w-4" />
          {cluster.key}
          <Badge variant="outline">{cluster.kind}</Badge>
        </CardTitle>
      </CardHeader>
      <CardContent class="space-y-1">
        {#each members as m, i (m.path)}
          <div class="flex items-center gap-2 text-sm">
            {#if i === 0}<Badge variant="default">newest</Badge>{/if}
            <span class="flex-1 truncate font-mono text-xs">{m.path}</span>
            <span class="text-xs text-muted-foreground">{ageLabel(m.ageDays)}</span>
            <span class="text-xs text-muted-foreground">{m.git.commits} commits</span>
            <Button size="sm" variant="ghost" onclick={() => onInspect(m)}>inspect</Button>
          </div>
        {/each}
      </CardContent>
    </Card>
  {:else}
    <Card>
      <CardContent class="py-8 text-center text-sm text-muted-foreground">
        No duplicate sets — nothing scanned shares a git remote or a folder name with anything else.
      </CardContent>
    </Card>
  {/each}
</div>
