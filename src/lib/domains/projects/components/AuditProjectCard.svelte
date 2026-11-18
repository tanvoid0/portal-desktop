<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import ShieldAlert from '~icons/fa6-solid/shield-halved';
  import HardDrive from '~icons/fa6-solid/hard-drive';
  import { fmtBytes } from '$lib/domains/disk/utils';
  import type { TriageDecision } from '$lib/domains/projects/api/discoveryApi';
  import type { DiscoveredProject } from '$lib/domains/projects/generated/DiscoveredProject';
  import { DECISIONS, DESTRUCTIVE_CLASS } from '$lib/domains/projects/utils/auditFormat';
  import AuditStatusBadge from './AuditStatusBadge.svelte';

  interface Props {
    project: DiscoveredProject;
    id: string;
    /** Keyboard cursor sits on this row. */
    isCursor: boolean;
    /** Details panel open. */
    isSelected: boolean;
    marked: boolean;
    decision: TriageDecision | null;
    onToggleMark: () => void;
    onSelect: () => void;
    onDecide: (d: TriageDecision) => void;
    onOpen: (abs: string) => void;
    onClean: () => void;
  }
  let {
    project: p,
    id,
    isCursor,
    isSelected,
    marked,
    decision,
    onToggleMark,
    onSelect,
    onDecide,
    onOpen,
    onClean,
  }: Props = $props();
</script>

<!--
  `tabindex=-1` so `move()` can put real focus here; the ring is a thickness
  change rather than a hue change, so the cursor is still visible without
  colour vision. Card draws no border of its own, so a `border-*` class would
  have set a colour on a zero-width border.
-->
<Card
  {id}
  tabindex={-1}
  aria-current={isCursor ? 'true' : undefined}
  class="audit-row outline-none focus-visible:ring-2 focus-visible:ring-ring {isCursor
    ? 'ring-2 ring-primary'
    : isSelected
      ? 'ring-1 ring-muted-foreground/30'
      : ''}"
>
  <CardContent class="flex flex-wrap items-center gap-2 p-3">
    <Checkbox checked={marked} onCheckedChange={onToggleMark} aria-label={`Select ${p.name}`} />
    <Button
      variant="ghost"
      class="h-auto min-w-0 flex-1 flex-col items-start justify-start p-1 text-left whitespace-normal"
      onclick={onSelect}
    >
      <div class="flex flex-wrap items-center gap-2">
        <span class="font-medium">{p.name}</span>
        {#if p.moduleCount > 1}
          <Badge variant="outline">{p.moduleCount} modules</Badge>
        {/if}
        {#each p.langs.slice(0, 3) as lang (lang)}
          <Badge variant="secondary">{lang}</Badge>
        {/each}
        {#if p.duplicateOf.length}
          <Badge variant="outline">dup: {p.duplicateOf[0]}</Badge>
        {/if}
      </div>
      <div class="w-full truncate font-mono text-xs text-muted-foreground">{p.path}</div>
    </Button>

    {#if p.regenerableBytes > 0}
      <Badge variant="outline" class="gap-1">
        <HardDrive class="h-3 w-3" />{fmtBytes(p.regenerableBytes)}
      </Badge>
    {/if}
    {#if p.riskLevel === 'high'}
      <Badge variant="destructive" class="gap-1">
        <ShieldAlert class="h-3 w-3" />{p.risk[0]}
      </Badge>
    {:else if p.risk.length}
      <Badge variant="outline">{p.risk[0]}</Badge>
    {/if}
    <AuditStatusBadge status={p.status} ageDays={p.ageDays} />

    <div class="flex gap-1">
      {#each DECISIONS as d (d)}
        <Button
          size="sm"
          variant={decision === d ? (d === 'delete' ? 'destructive' : 'default') : 'ghost'}
          class={d === 'delete' && decision !== d ? DESTRUCTIVE_CLASS : ''}
          onclick={() => onDecide(d)}
        >
          {d}
        </Button>
      {/each}
      <Button size="sm" variant="ghost" onclick={() => onOpen(p.abs)}>
        <FolderOpen class="h-3 w-3" />
      </Button>
    </div>
  </CardContent>

  {#if isSelected}
    <CardContent class="border-t pt-3 text-sm">
      {#if p.description}
        <p class="mb-2 text-muted-foreground">{p.description}</p>
      {/if}
      <div class="grid gap-1 md:grid-cols-2">
        <div>
          <span class="text-muted-foreground">Path:</span>
          <span class="font-mono text-xs">{p.abs}</span>
        </div>
        <div>
          <span class="text-muted-foreground">Git:</span>
          {p.git.repos
            ? `${p.git.repos} repo(s) · ${p.git.branch ?? '?'} · ${p.git.commits} commits`
            : 'not a repo'}
        </div>
        {#if p.git.remote}
          <div class="md:col-span-2">
            <span class="text-muted-foreground">Remote:</span>
            <span class="font-mono text-xs">{p.git.remote}</span>
          </div>
        {/if}
        {#if p.risk.length}
          <div class="md:col-span-2">
            <span class="text-muted-foreground">Flags:</span>
            {p.risk.join(' · ')}
          </div>
        {/if}
        {#if p.regenerable.length}
          <div class="flex flex-wrap items-center gap-2 md:col-span-2">
            <span class="text-muted-foreground">Build output:</span>
            <span>{p.regenerable.join(', ')}</span>
            {#if p.regenerableBytes >= 0}
              <Badge variant="outline">{fmtBytes(p.regenerableBytes)}</Badge>
            {/if}
            <Button size="sm" variant="outline" onclick={onClean}>
              <HardDrive class="mr-1 h-3 w-3" /> Clean in Disk utility
            </Button>
          </div>
        {/if}
      </div>

      {#if p.modules.length > 1}
        <div class="mt-3">
          <div class="mb-1 text-xs text-muted-foreground uppercase">
            Modules ({p.modules.length})
          </div>
          <div class="max-h-56 space-y-1 overflow-auto">
            {#each p.modules as m (m.path)}
              <div class="flex items-center gap-2 text-xs">
                <span class="flex-1 truncate font-mono">{m.path}</span>
                {#if m.ahead > 0}<Badge variant="destructive">+{m.ahead}</Badge>{/if}
                {#if m.git}<Badge variant="outline">{m.branch ?? 'git'}</Badge>{/if}
                <Button size="sm" variant="ghost" onclick={() => onOpen(m.abs)}>
                  <FolderOpen class="h-3 w-3" />
                </Button>
              </div>
            {/each}
          </div>
        </div>
      {/if}
    </CardContent>
  {/if}
</Card>
