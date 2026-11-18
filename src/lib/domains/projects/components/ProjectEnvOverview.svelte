<!-- "All" scope: one row per key, a badge per environment (file) that defines it. -->
<script lang="ts">
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '$lib/components/ui/table';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import type { EnvFileView } from '../generated';

  interface Props {
    files: EnvFileView[];
    onSelect: (fileName: string) => void;
  }

  const { files, onSelect }: Props = $props();

  const rows = $derived.by(() => {
    const byKey = new Map<string, { file: string; empty: boolean }[]>();
    for (const f of files)
      for (const v of f.vars) {
        const list = byKey.get(v.key) ?? [];
        list.push({ file: f.name, empty: v.value === '' });
        byKey.set(v.key, list);
      }
    return [...byKey.entries()].sort(([a], [b]) => a.localeCompare(b));
  });
</script>

<Table class="table-fixed">
  <TableHeader>
    <TableRow>
      <TableHead class="w-[40%]">Key</TableHead>
      <TableHead>Environments</TableHead>
    </TableRow>
  </TableHeader>
  <TableBody>
    {#each rows as [key, envs] (key)}
      <TableRow>
        <TableCell class="truncate font-mono" title={key}>{key}</TableCell>
        <TableCell>
          <div class="flex flex-wrap gap-1.5">
            {#each envs as e (e.file)}
              <Button
                variant="ghost"
                class="h-auto p-0 hover:bg-transparent"
                onclick={() => onSelect(e.file)}
                title={e.empty ? `${e.file} (empty value)` : e.file}
              >
                <Badge variant={e.empty ? 'outline' : 'secondary'} class="font-mono">
                  {e.file}{e.empty ? ' · empty' : ''}
                </Badge>
              </Button>
            {/each}
          </div>
        </TableCell>
      </TableRow>
    {/each}
  </TableBody>
</Table>
