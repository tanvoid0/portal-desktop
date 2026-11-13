<script lang="ts">
  import * as Table from '$lib/components/ui/table';
  import type { ColumnInfo, QueryResult } from '../generated';

  interface Props {
    result: QueryResult;
    columnInfo?: ColumnInfo[];
  }
  let { result, columnInfo = [] }: Props = $props();

  function info(name: string): ColumnInfo | undefined {
    return columnInfo.find((x) => x.name === name);
  }
  function hint(name: string): string | undefined {
    const c = info(name);
    if (!c) return undefined;
    return [c.dataType, c.pk ? 'PK' : '', c.notNull ? 'NOT NULL' : ''].filter(Boolean).join(' · ');
  }
</script>

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
      </Table.Row>
    </Table.Header>
    <Table.Body>
      {#each result.rows as row, i (i)}
        <Table.Row>
          {#each row as cell, j (j)}
            <Table.Cell class="max-w-xs truncate font-mono text-xs">
              {#if cell === null}
                <span class="text-muted-foreground">NULL</span>
              {:else if typeof cell === 'object'}
                {JSON.stringify(cell)}
              {:else}
                {String(cell)}
              {/if}
            </Table.Cell>
          {/each}
        </Table.Row>
      {/each}
    </Table.Body>
  </Table.Root>
</div>
{#if result.truncated}
  <p class="text-xs text-muted-foreground">Showing first {result.rowCount} rows</p>
{/if}
