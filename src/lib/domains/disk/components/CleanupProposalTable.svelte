<script lang="ts">
  import type { ItemVerdict } from '../types';
  import { fmtBytes, leafIds, RISK_BADGE, VERDICT_BADGE, type TreeNode } from '../utils';
  import { Card } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '$lib/components/ui/table';
  import CollapseToggle from './shared/CollapseToggle.svelte';

  let {
    rows,
    selected,
    expanded,
    verdictByPath,
    empty,
    ontoggleSel,
    ontoggleExp,
  }: {
    rows: { node: TreeNode; depth: number; isFolder: boolean }[];
    selected: Set<string>;
    expanded: Set<string>;
    verdictByPath: Map<string, ItemVerdict>;
    empty: boolean;
    ontoggleSel: (ids: string[], on: boolean) => void;
    ontoggleExp: (path: string) => void;
  } = $props();

  function rowSel(node: TreeNode) {
    const ids = leafIds(node);
    const sel = ids.filter((id) => selected.has(id)).length;
    return { ids, all: sel === ids.length && ids.length > 0, some: sel > 0 && sel !== ids.length };
  }
</script>

<Card class="gap-0 overflow-hidden py-0">
  <Table>
    <TableHeader>
      <TableRow>
        <TableHead class="w-8"></TableHead>
        <TableHead>Path</TableHead>
        <TableHead>Kind</TableHead>
        <TableHead>Reason</TableHead>
        <TableHead class="text-right">Size</TableHead>
        <TableHead>Risk</TableHead>
        {#if verdictByPath.size > 0}<TableHead>AI</TableHead>{/if}
      </TableRow>
    </TableHeader>
    <TableBody>
      {#each rows as { node, depth, isFolder } (node.fullPath + (node.proposal?.id ?? ''))}
        {@const sel = rowSel(node)}
        {@const p = node.proposal}
        <TableRow>
          <TableCell class="align-top">
            <Checkbox
              checked={sel.all}
              indeterminate={sel.some}
              onCheckedChange={() => ontoggleSel(sel.ids, !sel.all)}
            />
          </TableCell>
          <TableCell class="align-top font-mono text-xs break-all">
            <div class="flex items-start" style="padding-left: {depth * 16}px">
              {#if isFolder}
                <CollapseToggle
                  collapsed={!expanded.has(node.fullPath)}
                  class="mt-px mr-1.5"
                  onclick={() => ontoggleExp(node.fullPath)}
                />
              {:else}
                <span class="mr-1.5 w-4 shrink-0 text-muted-foreground/50">·</span>
              {/if}
              <span class={isFolder ? 'text-foreground' : ''}>
                {node.name}
                {#if isFolder}<span class="ml-2 text-muted-foreground"
                    >{node.count} item{node.count === 1 ? '' : 's'}</span
                  >{/if}
              </span>
            </div>
          </TableCell>
          <TableCell class="align-top text-muted-foreground">{p ? p.kind : ''}</TableCell>
          <TableCell class="align-top text-muted-foreground">{p ? p.reason : ''}</TableCell>
          <TableCell class="text-right align-top whitespace-nowrap"
            >{fmtBytes(node.sizeBytes)}</TableCell
          >
          <TableCell class="align-top">
            {#if p}<Badge variant="outline" class={RISK_BADGE[p.risk]}>{p.risk}</Badge>{/if}
          </TableCell>
          {#if verdictByPath.size > 0}
            <TableCell class="align-top">
              {#if p && verdictByPath.has(p.path)}
                {@const v = verdictByPath.get(p.path)!}
                <Badge variant="outline" class={VERDICT_BADGE[v.verdict]} title={v.reason}
                  >{v.verdict}</Badge
                >
              {/if}
            </TableCell>
          {/if}
        </TableRow>
      {/each}
      {#if empty}
        <TableRow>
          <TableCell
            colspan={verdictByPath.size > 0 ? 7 : 6}
            class="py-10 text-center text-muted-foreground"
          >
            No cleanup proposals. Disk looks tidy.
          </TableCell>
        </TableRow>
      {/if}
    </TableBody>
  </Table>
</Card>
