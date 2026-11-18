<script lang="ts">
  import type { DevCleanerGroup, DevCleanerItem } from '../types';
  import { fmtBytes, KIND_BADGE, RISK_BADGE } from '../utils';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Table, TableBody, TableCell, TableRow } from '$lib/components/ui/table';
  import CollapseToggle from './shared/CollapseToggle.svelte';

  let {
    group: g,
    selected,
    collapsed,
    subCollapsed,
    ontoggleSel,
    ontoggleCollapse,
    ontoggleSub,
  }: {
    group: DevCleanerGroup;
    selected: Set<string>;
    collapsed: Set<string>;
    subCollapsed: Set<string>;
    ontoggleSel: (ids: string[], on: boolean) => void;
    ontoggleCollapse: (groupId: string) => void;
    ontoggleSub: (label: string) => void;
  } = $props();

  function projectSubgroups(items: DevCleanerItem[]) {
    const map = new Map<string, DevCleanerItem[]>();
    for (const item of items) {
      const key = item.groupLabel ?? item.path;
      const list = map.get(key) ?? [];
      list.push(item);
      map.set(key, list);
    }
    return [...map.entries()]
      .map(([label, subItems]) => ({
        label,
        items: subItems,
        bytes: subItems.reduce((a, i) => a + i.sizeBytes, 0),
      }))
      .sort((a, b) => b.bytes - a.bytes);
  }

  function groupSel(items: DevCleanerItem[]) {
    const ids = items.map((i) => i.id);
    const sel = ids.filter((id) => selected.has(id)).length;
    return { ids, all: sel === ids.length && ids.length > 0, some: sel > 0 && sel !== ids.length };
  }

  function kindLabel(kind: string): string {
    return kind.replace(/-/g, ' ');
  }
</script>

{#snippet itemCheck(item: DevCleanerItem)}
  <TableCell class="w-8">
    <Checkbox
      checked={selected.has(item.id)}
      onCheckedChange={() => ontoggleSel([item.id], !selected.has(item.id))}
    />
  </TableCell>
{/snippet}

{#snippet riskReasonSize(item: DevCleanerItem)}
  <TableCell>
    <Badge variant="outline" class={RISK_BADGE[item.risk] ?? ''}>{item.risk}</Badge>
  </TableCell>
  <TableCell class="max-w-xs truncate text-xs text-muted-foreground" title={item.reason}>
    {item.reason}
  </TableCell>
  <TableCell class="text-right whitespace-nowrap">{fmtBytes(item.sizeBytes)}</TableCell>
{/snippet}

<Card class="gap-0 overflow-hidden py-0">
  <div class="divider-edge-b divider-edge-full flex items-center gap-2.5 bg-muted/40 px-3 py-2.5">
    {#if g.available && g.items.length > 0}
      {@const sel = groupSel(g.items)}
      <Checkbox
        checked={sel.all}
        indeterminate={sel.some}
        onCheckedChange={() => ontoggleSel(sel.ids, !sel.all)}
      />
    {:else}
      <span class="w-4"></span>
    {/if}
    <CollapseToggle
      collapsed={collapsed.has(g.cleanerId)}
      onclick={() => ontoggleCollapse(g.cleanerId)}
    />
    <Badge variant="outline" class={KIND_BADGE[g.cleanerId] ?? ''}>{g.label}</Badge>
    {#if !g.available}
      <span class="text-xs text-muted-foreground">{g.unavailableReason ?? 'not available'}</span>
    {:else}
      <div class="ml-auto flex items-center gap-3 whitespace-nowrap">
        <span class="text-xs text-muted-foreground"
          >{g.items.length} item{g.items.length === 1 ? '' : 's'}</span
        >
        <span class="text-sm font-semibold text-foreground tabular-nums"
          >{fmtBytes(g.totalBytes)}</span
        >
      </div>
    {/if}
  </div>
  {#if !collapsed.has(g.cleanerId) && g.available}
    {#if g.items.length === 0}
      <CardContent class="py-6 text-center text-sm text-muted-foreground">
        Nothing to reclaim.
      </CardContent>
    {:else if g.cleanerId === 'projects'}
      {#each projectSubgroups(g.items) as sub (sub.label)}
        {@const sel = groupSel(sub.items)}
        <div class="divider-edge-b divider-edge-full last:[&::after]:hidden">
          <div class="flex items-center gap-2 bg-muted/20 px-3 py-2">
            <Checkbox
              checked={sel.all}
              indeterminate={sel.some}
              onCheckedChange={() => ontoggleSel(sel.ids, !sel.all)}
            />
            <CollapseToggle
              collapsed={subCollapsed.has(sub.label)}
              onclick={() => ontoggleSub(sub.label)}
            />
            <span class="font-mono text-xs break-all text-foreground" title={sub.label}>
              {sub.label}
            </span>
            <span class="ml-auto flex items-center gap-3 whitespace-nowrap">
              <span class="text-xs text-muted-foreground">
                {sub.items.length} dir{sub.items.length === 1 ? '' : 's'}
              </span>
              <span class="text-sm font-semibold tabular-nums">{fmtBytes(sub.bytes)}</span>
            </span>
          </div>
          {#if !subCollapsed.has(sub.label)}
            <Table>
              <TableBody>
                {#each sub.items as item (item.id)}
                  <TableRow>
                    {@render itemCheck(item)}
                    <TableCell class="font-mono text-xs">
                      <span class="text-muted-foreground">…\</span>{item.kind}
                    </TableCell>
                    <TableCell class="text-xs whitespace-nowrap text-muted-foreground">
                      {item.fileCount.toLocaleString()} files
                    </TableCell>
                    {@render riskReasonSize(item)}
                  </TableRow>
                {/each}
              </TableBody>
            </Table>
          {/if}
        </div>
      {/each}
    {:else}
      <Table>
        <TableBody>
          {#each g.items as item (item.id)}
            <TableRow>
              {@render itemCheck(item)}
              <TableCell class="font-mono text-xs break-all text-foreground">{item.path}</TableCell>
              <TableCell>
                <Badge variant="outline" class={KIND_BADGE[item.kind] ?? ''}>
                  {kindLabel(item.kind)}
                </Badge>
              </TableCell>
              {@render riskReasonSize(item)}
            </TableRow>
          {/each}
        </TableBody>
      </Table>
    {/if}
  {/if}
</Card>
