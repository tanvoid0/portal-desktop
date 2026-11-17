<script lang="ts">
  import { createInfiniteQuery, createQuery, useQueryClient } from '@tanstack/svelte-query';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
  import * as Table from '$lib/components/ui/table';
  import { PageLoading, PageError } from '$lib/components/shell';
  import {
    AlertDialog,
    AlertDialogAction,
    AlertDialogCancel,
    AlertDialogContent,
    AlertDialogFooter,
    AlertDialogHeader,
    AlertDialogTitle,
  } from '$lib/components/ui/alert-dialog';
  import { toast } from '$lib/utils/toast';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { redisGet, redisPreview, redisScan, redisWrite } from '../api/databaseApi';
  import type { RedisWrite } from '../generated';

  interface Props {
    sourceId: string;
    /** Logical database index. */
    db: number;
    editable?: boolean;
  }
  let { sourceId, db, editable = false }: Props = $props();

  const client = useQueryClient();

  let patternText = $state('');
  let pattern = $state(''); // the applied pattern (set on Enter)
  let selected = $state<string | null>(null);
  let creating = $state(false); // the New key form (a separate flag: "new" is a legal key)
  let draft = $state('');
  let ttlText = $state('');
  let newKey = $state('');

  const keys = createInfiniteQuery(() => ({
    queryKey: queryKeys.database.redisKeys(sourceId, db, pattern),
    queryFn: ({ pageParam }) => redisScan(sourceId, db, pattern, pageParam),
    initialPageParam: '0',
    getNextPageParam: (last) => (last.cursor === '0' ? undefined : last.cursor),
    retry: false,
  }));
  // SCAN can repeat a key across pages.
  const keyList = $derived([...new Set(keys.data?.pages.flatMap((p) => p.keys) ?? [])]);

  const value = createQuery(() => ({
    queryKey: queryKeys.database.redisValue(sourceId, db, selected ?? ''),
    queryFn: () => redisGet(sourceId, db, selected!),
    enabled: selected !== null,
    retry: false,
    refetchOnWindowFocus: false, // would clobber an unsaved draft
  }));

  // Reset the editor whenever a (re)loaded value arrives.
  $effect(() => {
    const v = value.data;
    if (!v) return;
    draft = v.text ?? '';
    // TTL 0 = under a second left; "empty" would mean no expiry, so floor at 1.
    ttlText = v.ttl >= 0 ? String(Math.max(v.ttl, 1)) : '';
  });

  function applyPattern() {
    pattern = patternText.trim();
    selected = null;
  }

  function pick(k: string) {
    creating = false;
    selected = k;
  }

  function startCreate() {
    creating = true;
    selected = null;
    newKey = '';
    draft = '';
    ttlText = '';
  }

  // Own dialog (not confirmAction): the command text can be long, so it scrolls
  // in a <pre> and Confirm stays on screen.
  let confirm = $state<{
    text: string;
    title: string;
    write: RedisWrite;
    done: string;
    destructive: boolean;
  } | null>(null);
  let confirmOpen = $state(false);

  async function ask(write: RedisWrite, title: string, done: string, destructive = false) {
    try {
      const text = await redisPreview(sourceId, write);
      confirm = { text, title: `${title} · db ${db}`, write, done, destructive };
      confirmOpen = true;
    } catch (e) {
      toast.error(`${title} failed`, String(e));
    }
  }

  async function proceed() {
    const c = confirm;
    confirmOpen = false; // AlertDialogAction does not close the dialog by itself
    if (!c) return;
    try {
      await redisWrite(sourceId, db, c.write);
      toast.success(c.done);
      if (c.write.op === 'del' || creating) {
        selected = null;
        creating = false;
      }
      client.invalidateQueries({ queryKey: queryKeys.database.redisDb(sourceId, db) });
      client.invalidateQueries({ queryKey: queryKeys.database.redisDatabases(sourceId) });
    } catch (e) {
      toast.error(`${c.title} failed`, String(e));
    }
  }

  /** Empty = no expiry; otherwise whole seconds (0 and junk are rejected by the backend). */
  function ttl(): number | null | undefined {
    const t = String(ttlText ?? '').trim();
    if (t === '') return null;
    if (!/^\d+$/.test(t)) {
      toast.error('Invalid TTL', 'Whole seconds, or empty for no expiry');
      return undefined;
    }
    return Number(t);
  }

  function save() {
    const t = ttl();
    if (t === undefined || (!creating && selected === null)) return;
    const create = creating;
    const key = create ? newKey : selected!;
    ask(
      { op: 'set', key, value: draft, ttl: t, create },
      create ? 'New key' : 'Set key',
      create ? 'Key created' : 'Key saved'
    );
  }

  function remove() {
    if (selected === null) return;
    ask({ op: 'del', key: selected }, 'Delete key', 'Key deleted', true);
  }

  const TTL_HINT = 'TTL counts from Save; empty = no expiry.';

  const HEADERS: Record<string, string[]> = {
    hash: ['Field', 'Value'],
    list: ['#', 'Value'],
    set: ['Member'],
    zset: ['Member', 'Score'],
  };
</script>

<div class="space-y-3">
  <div class="flex flex-wrap items-center gap-2">
    <Input
      bind:value={patternText}
      class="min-w-48 flex-1 font-mono text-sm"
      placeholder="Pattern, e.g. user:* — Enter to apply"
      onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && applyPattern()}
    />
    {#if editable}
      <Button variant="outline" size="sm" onclick={startCreate}>New key</Button>
    {/if}
  </div>

  {#if keys.isPending}
    <PageLoading message="Scanning keys..." />
  {:else if keys.isError}
    <PageError title="Scan failed" message={String(keys.error)} onRetry={() => keys.refetch()} />
  {:else}
    <div class="grid grid-cols-1 gap-3 lg:grid-cols-[minmax(0,16rem)_minmax(0,1fr)]">
      <div class="max-h-[calc(100vh-18rem)] space-y-1 overflow-y-auto rounded-md border p-1">
        {#each keyList as k (k)}
          <Button
            variant={selected === k ? 'secondary' : 'ghost'}
            size="sm"
            class="h-7 w-full justify-start font-mono text-xs"
            onclick={() => pick(k)}
          >
            <span class="truncate">{k}</span>
          </Button>
        {:else}
          <p class="p-2 text-sm text-muted-foreground">No keys.</p>
        {/each}
        {#if keys.hasNextPage}
          <Button
            variant="outline"
            size="sm"
            class="w-full"
            disabled={keys.isFetchingNextPage}
            onclick={() => keys.fetchNextPage()}
          >
            Load more
          </Button>
        {/if}
      </div>

      <div class="min-w-0 space-y-2">
        {#if creating}
          <Input bind:value={newKey} class="font-mono text-sm" placeholder="Key" />
          <Textarea
            bind:value={draft}
            rows={10}
            class="font-mono text-xs"
            spellcheck={false}
            placeholder="Value"
          />
          <div class="flex items-center gap-2">
            <Input
              inputmode="numeric"
              bind:value={ttlText}
              class="w-40"
              placeholder="TTL (s), empty = none"
            />
            <Button size="sm" onclick={save} disabled={!newKey}>Create</Button>
          </div>
          <p class="text-xs text-muted-foreground">{TTL_HINT}</p>
        {:else if selected === null}
          <p class="text-sm text-muted-foreground">Select a key.</p>
        {:else if value.isPending}
          <PageLoading message="Loading value..." />
        {:else if value.isError}
          <PageError
            title="Could not load key"
            message={String(value.error)}
            onRetry={() => value.refetch()}
          />
        {:else}
          {@const v = value.data}
          <div class="flex flex-wrap items-center gap-2 text-xs text-muted-foreground">
            <Badge variant="outline">{v.kind}</Badge>
            <span>{v.ttl < 0 ? 'no expiry' : `${v.ttl}s`}</span>
            <span class="tabular-nums">len {v.len}</span>
            {#if v.truncated}
              <span>
                showing first {v.kind === 'string' ? '1 MiB' : v.items.length} of {v.len}
              </span>
            {/if}
          </div>

          {@const noEdit = v.binary || v.truncated || (v.text ?? '').includes('\r')}
          {#if v.kind === 'string'}
            <Textarea
              bind:value={draft}
              rows={14}
              readonly={!editable || noEdit}
              class="font-mono text-xs"
              spellcheck={false}
            />
            {#if v.binary}
              <p class="text-xs text-muted-foreground">Binary value, shown lossy — not editable.</p>
            {:else if v.truncated}
              <p class="text-xs text-muted-foreground">Truncated value — not editable.</p>
            {:else if noEdit}
              <p class="text-xs text-muted-foreground">
                Contains carriage returns (\r), which the editor would strip — not editable.
              </p>
            {/if}
          {:else if HEADERS[v.kind]}
            <Table.Root>
              <Table.Header>
                <Table.Row>
                  {#each HEADERS[v.kind] as h (h)}
                    <Table.Head>{h}</Table.Head>
                  {/each}
                </Table.Row>
              </Table.Header>
              <Table.Body>
                {#each v.items as row, i (i)}
                  <Table.Row>
                    {#if v.kind === 'list'}
                      <Table.Cell class="tabular-nums">{i}</Table.Cell>
                    {/if}
                    {#each row as cell, j (j)}
                      <Table.Cell class="max-w-md font-mono text-xs break-all whitespace-pre-wrap">
                        {cell}
                      </Table.Cell>
                    {/each}
                  </Table.Row>
                {/each}
              </Table.Body>
            </Table.Root>
          {:else}
            <p class="text-sm text-muted-foreground">
              {v.kind} values are not supported for viewing
            </p>
          {/if}

          {#if editable}
            <div class="flex flex-wrap items-center gap-2">
              {#if v.kind === 'string' && !noEdit}
                <Input
                  inputmode="numeric"
                  bind:value={ttlText}
                  class="w-40"
                  placeholder="TTL (s), empty = none"
                />
                <Button size="sm" onclick={save}>Save</Button>
              {/if}
              <Button variant="destructive" size="sm" onclick={remove}>Delete</Button>
            </div>
            {#if v.kind === 'string' && !noEdit}
              <p class="text-xs text-muted-foreground">{TTL_HINT}</p>
            {/if}
          {/if}
        {/if}
      </div>
    </div>
  {/if}
</div>

<AlertDialog bind:open={confirmOpen}>
  <AlertDialogContent class="sm:max-w-2xl">
    <AlertDialogHeader>
      <AlertDialogTitle>{confirm?.title}</AlertDialogTitle>
    </AlertDialogHeader>
    <pre
      class="max-h-[50vh] overflow-auto rounded-md border bg-muted p-2 text-xs break-all whitespace-pre-wrap">{confirm?.text}</pre>
    <AlertDialogFooter>
      <AlertDialogCancel>Cancel</AlertDialogCancel>
      <AlertDialogAction
        class={confirm?.destructive ? 'bg-destructive text-white hover:bg-destructive/90' : ''}
        onclick={proceed}>Confirm</AlertDialogAction
      >
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
