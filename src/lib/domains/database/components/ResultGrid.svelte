<script lang="ts">
  import * as Table from '$lib/components/ui/table';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import Plus from '~icons/fa6-solid/plus';
  import Trash from '~icons/fa6-solid/trash';
  import { confirmAction } from '$lib/utils/confirm';
  import { toast } from '$lib/utils/toast';
  import {
    applyDelete,
    applyInsert,
    applyUpdate,
    previewDelete,
    previewInsert,
    previewUpdate,
  } from '../api/databaseApi';
  import type { ColumnInfo, QueryResult } from '../generated';

  interface Props {
    result: QueryResult;
    columnInfo?: ColumnInfo[];
    /** Editing needs all three: the source toggled on, a sourceId/table to act
     *  on, and a detectable PK (rows with none stay read-only). */
    editable?: boolean;
    sourceId?: string;
    table?: string;
    /** Called after a successful edit so the caller refetches the page. */
    onChanged?: () => void;
  }
  let {
    result,
    columnInfo = [],
    editable = false,
    sourceId,
    table,
    onChanged = () => {},
  }: Props = $props();

  function info(name: string): ColumnInfo | undefined {
    return columnInfo.find((x) => x.name === name);
  }
  function hint(name: string): string | undefined {
    const c = info(name);
    if (!c) return undefined;
    return [c.dataType, c.pk ? 'PK' : '', c.notNull ? 'NOT NULL' : ''].filter(Boolean).join(' · ');
  }

  const pkColumn = $derived(columnInfo.find((c) => c.pk)?.name);
  const pkIndex = $derived(pkColumn ? result.columns.indexOf(pkColumn) : -1);
  const canEdit = $derived(Boolean(editable && sourceId && table && pkColumn && pkIndex >= 0));

  // Back to the original cell's JS type so an edited string round-trips as a
  // number/boolean instead of always becoming text.
  function toJsonValue(raw: string, original: unknown): unknown {
    if (raw === '') return null;
    if (typeof original === 'number') {
      const n = Number(raw);
      return Number.isNaN(n) ? raw : n;
    }
    if (typeof original === 'boolean') return raw === 'true';
    return raw;
  }

  function previewMessage(sql: string, params: unknown[]): string {
    return params.length ? `${sql}\n\nvalues: ${JSON.stringify(params)}` : sql;
  }

  let editingCell = $state<{ row: number; col: number } | null>(null);
  let editValue = $state('');

  function startEdit(rowIdx: number, colIdx: number) {
    if (!canEdit) return;
    if (editingCell?.row === rowIdx && editingCell?.col === colIdx) return; // already editing this cell
    editingCell = { row: rowIdx, col: colIdx };
    const v = result.rows[rowIdx][colIdx];
    editValue = v === null ? '' : String(v);
  }

  async function commitEdit(rowIdx: number, colIdx: number) {
    // Re-entrant: Enter commits then clears editingCell, which unmounts the
    // input and fires a synchronous blur -> a second commitEdit call for the
    // same cell. Escape clears editingCell the same way. Both no-op here.
    if (editingCell?.row !== rowIdx || editingCell?.col !== colIdx) return;
    const original = result.rows[rowIdx][colIdx];
    const raw = editValue;
    editingCell = null;
    if ((original === null ? '' : String(original)) === raw) return;

    const newValue = toJsonValue(raw, original);
    const pkValue = result.rows[rowIdx][pkIndex];
    const column = result.columns[colIdx];
    try {
      const preview = await previewUpdate(sourceId!, table!, pkColumn!, pkValue, column, newValue);
      const ok = await confirmAction(previewMessage(preview.sql, preview.params), 'Confirm update');
      if (!ok) return;
      await applyUpdate(sourceId!, table!, pkColumn!, pkValue, column, newValue);
      toast.success('Row updated');
      onChanged();
    } catch (e) {
      toast.error('Update failed', String(e));
    }
  }

  async function deleteRow(rowIdx: number) {
    const pkValue = result.rows[rowIdx][pkIndex];
    try {
      const preview = await previewDelete(sourceId!, table!, pkColumn!, pkValue);
      const ok = await confirmAction(previewMessage(preview.sql, preview.params), 'Delete row', {
        destructive: true,
      });
      if (!ok) return;
      await applyDelete(sourceId!, table!, pkColumn!, pkValue);
      toast.success('Row deleted');
      onChanged();
    } catch (e) {
      toast.error('Delete failed', String(e));
    }
  }

  let inserting = $state(false);
  let insertValues = $state<Record<string, string>>({});

  function startInsert() {
    insertValues = Object.fromEntries(result.columns.map((c) => [c, '']));
    inserting = true;
  }

  async function commitInsert() {
    const values: [string, unknown][] = result.columns
      .filter((c) => insertValues[c] !== '')
      .map((c) => [c, insertValues[c]]);
    try {
      const preview = await previewInsert(sourceId!, table!, values);
      const ok = await confirmAction(previewMessage(preview.sql, preview.params), 'Insert row');
      if (!ok) return;
      await applyInsert(sourceId!, table!, values);
      inserting = false;
      toast.success('Row inserted');
      onChanged();
    } catch (e) {
      toast.error('Insert failed', String(e));
    }
  }
</script>

{#if canEdit}
  <div class="flex justify-end">
    <Button variant="outline" size="sm" onclick={startInsert} disabled={inserting}>
      <Plus class="h-3.5 w-3.5" /> Insert row
    </Button>
  </div>
{/if}

<div class="max-h-[60vh] overflow-auto rounded-md border">
  <Table.Root>
    <Table.Header>
      <Table.Row>
        {#each result.columns as col (col)}
          <Table.Head title={hint(col)} class="whitespace-nowrap">
            {col}
            {#if info(col)?.pk}
              <span class="text-xs text-primary">PK</span>
            {/if}
          </Table.Head>
        {/each}
        {#if canEdit}
          <Table.Head class="w-10"></Table.Head>
        {/if}
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#if inserting}
        <Table.Row>
          {#each result.columns as col (col)}
            <Table.Cell class="max-w-xs">
              <Input
                bind:value={insertValues[col]}
                placeholder={col}
                class="h-7 font-mono text-xs"
              />
            </Table.Cell>
          {/each}
          <Table.Cell class="whitespace-nowrap">
            <Button variant="ghost" size="sm" class="h-7" onclick={commitInsert}>Save</Button>
            <Button variant="ghost" size="sm" class="h-7" onclick={() => (inserting = false)}
              >Cancel</Button
            >
          </Table.Cell>
        </Table.Row>
      {/if}
      {#each result.rows as row, i (i)}
        <Table.Row>
          {#each row as cell, j (j)}
            <Table.Cell
              class="max-w-xs truncate font-mono text-xs {canEdit ? 'cursor-pointer' : ''}"
              onclick={() => startEdit(i, j)}
            >
              {#if editingCell?.row === i && editingCell?.col === j}
                <Input
                  bind:value={editValue}
                  autofocus
                  class="h-7 font-mono text-xs"
                  onblur={() => commitEdit(i, j)}
                  onkeydown={(e) => {
                    if (e.key === 'Enter') commitEdit(i, j);
                    if (e.key === 'Escape') editingCell = null;
                  }}
                />
              {:else if cell === null}
                <span class="text-muted-foreground">NULL</span>
              {:else if typeof cell === 'object'}
                {JSON.stringify(cell)}
              {:else}
                {String(cell)}
              {/if}
            </Table.Cell>
          {/each}
          {#if canEdit}
            <Table.Cell class="whitespace-nowrap">
              <Button
                variant="ghost"
                size="sm"
                class="h-7 w-7 p-0 text-destructive"
                onclick={() => deleteRow(i)}
              >
                <Trash class="h-3.5 w-3.5" />
              </Button>
            </Table.Cell>
          {/if}
        </Table.Row>
      {/each}
    </Table.Body>
  </Table.Root>
</div>
{#if result.truncated}
  <p class="text-xs text-muted-foreground">Showing first {result.rowCount} rows</p>
{/if}
