<script lang="ts">
  import { createQuery, useQueryClient } from '@tanstack/svelte-query';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Textarea } from '$lib/components/ui/textarea';
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
  import { mongoDelete, mongoFind, mongoInsert, mongoReplace } from '../api/databaseApi';
  import type { MongoDoc } from '../generated';

  interface Props {
    sourceId: string;
    db: string;
    collection: string;
    editable?: boolean;
  }
  let { sourceId, db, collection, editable = false }: Props = $props();

  const PAGE_SIZE = 50;
  const client = useQueryClient();

  let filterText = $state('');
  let filter = $state(''); // the applied filter (set on Enter)
  let page = $state(0);
  // raw: a deep proxy would break the `selected === d` highlight check.
  let selected = $state.raw<MongoDoc | 'new' | null>(null);
  let draft = $state('');

  const docs = createQuery(() => ({
    queryKey: queryKeys.database.mongoDocs(sourceId, db, collection, filter, page),
    queryFn: () => mongoFind(sourceId, db, collection, filter, PAGE_SIZE, page * PAGE_SIZE),
    retry: false,
  }));

  const ns = $derived(`${db}.${collection}`);
  const idLabel = (d: MongoDoc) => d.id?.replace(/^\{"\$oid":"(.*)"\}$/, '$1') ?? '(no _id)';

  function applyFilter() {
    filter = filterText.trim();
    page = 0;
    selected = null;
  }

  function pick(d: MongoDoc | 'new') {
    selected = d;
    draft = d === 'new' ? '{\n  \n}' : d.json;
  }

  /** JSON.parse only validates; the exact draft text is what gets sent. */
  function validJson(): boolean {
    try {
      JSON.parse(draft);
      return true;
    } catch (e) {
      toast.error('Invalid JSON', String(e));
      return false;
    }
  }

  // Own dialog (not confirmAction): a document can be KBs, so the op text
  // scrolls in a <pre> and Confirm stays on screen.
  let confirm = $state<{
    text: string;
    title: string;
    op: () => Promise<unknown>;
    done: string;
    destructive: boolean;
  } | null>(null);
  let confirmOpen = $state(false);

  function ask(
    text: string,
    title: string,
    op: () => Promise<unknown>,
    done: string,
    destructive = false
  ) {
    confirm = { text, title, op, done, destructive };
    confirmOpen = true;
  }

  let busy = $state(false);

  async function proceed() {
    const c = confirm;
    if (busy) return;
    confirmOpen = false; // AlertDialogAction does not close the dialog by itself
    if (!c) return;
    busy = true;
    try {
      await c.op();
      toast.success(c.done);
      selected = null;
      client.invalidateQueries({
        queryKey: queryKeys.database.mongoCollection(sourceId, db, collection),
      });
    } catch (e) {
      toast.error(`${c.title} failed`, String(e));
    } finally {
      busy = false;
    }
  }

  function save() {
    if (selected === null || !validJson()) return;
    if (selected === 'new') {
      ask(
        `${ns}.insertOne(${draft})`,
        'Insert document',
        () => mongoInsert(sourceId, db, collection, draft),
        'Document inserted'
      );
    } else {
      const id = selected.id!;
      ask(
        `${ns}.replaceOne({_id: ${id}}, ${draft})`,
        'Replace document',
        () => mongoReplace(sourceId, db, collection, id, draft),
        'Document saved'
      );
    }
  }

  function remove() {
    if (selected === null || selected === 'new') return;
    const id = selected.id!;
    ask(
      `${ns}.deleteOne({_id: ${id}})`,
      'Delete document',
      () => mongoDelete(sourceId, db, collection, id),
      'Document deleted',
      true
    );
  }
</script>

<div class="space-y-3">
  <div class="flex flex-wrap items-center gap-2">
    <Input
      bind:value={filterText}
      class="min-w-48 flex-1 font-mono text-sm"
      placeholder={'Filter, e.g. {"status": "active"} — Enter to apply'}
      onkeydown={(e: KeyboardEvent) => e.key === 'Enter' && applyFilter()}
    />
    {#if docs.data}
      <span class="text-xs text-muted-foreground tabular-nums">{docs.data.total} documents</span>
    {/if}
    {#if editable}
      <Button variant="outline" size="sm" onclick={() => pick('new')}>New document</Button>
    {/if}
  </div>

  {#if docs.isPending}
    <PageLoading message="Loading documents..." />
  {:else if docs.isError}
    <PageError title="Query failed" message={String(docs.error)} onRetry={() => docs.refetch()} />
  {:else}
    <div class="grid grid-cols-1 gap-3 lg:grid-cols-[minmax(0,14rem)_minmax(0,1fr)]">
      <div class="max-h-[calc(100vh-18rem)] space-y-1 overflow-y-auto rounded-md border p-1">
        {#each docs.data.docs as d, i (d.id ?? i)}
          <Button
            variant={selected === d ? 'secondary' : 'ghost'}
            size="sm"
            class="h-7 w-full justify-start font-mono text-xs"
            onclick={() => pick(d)}
          >
            <span class="truncate">{idLabel(d)}</span>
          </Button>
        {:else}
          <p class="p-2 text-sm text-muted-foreground">No documents.</p>
        {/each}
      </div>

      <div class="min-w-0 space-y-2">
        {#if selected === null}
          <p class="text-sm text-muted-foreground">Select a document.</p>
        {:else}
          <Textarea
            bind:value={draft}
            rows={18}
            readonly={!editable}
            class="font-mono text-xs"
            spellcheck={false}
          />
          {#if editable}
            <div class="flex gap-2">
              <Button size="sm" onclick={save} disabled={selected !== 'new' && !selected.id}>
                {selected === 'new' ? 'Insert' : 'Save'}
              </Button>
              {#if selected !== 'new'}
                <Button variant="destructive" size="sm" onclick={remove} disabled={!selected.id}>
                  Delete
                </Button>
              {/if}
            </div>
          {/if}
        {/if}
      </div>
    </div>

    <div class="flex items-center gap-2">
      <Button variant="outline" size="sm" disabled={page === 0} onclick={() => (page -= 1)}>
        Prev
      </Button>
      <span class="text-xs text-muted-foreground tabular-nums">Page {page + 1}</span>
      <Button
        variant="outline"
        size="sm"
        disabled={(page + 1) * PAGE_SIZE >= docs.data.total}
        onclick={() => (page += 1)}
      >
        Next
      </Button>
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
        disabled={busy}
        onclick={proceed}>Confirm</AlertDialogAction
      >
    </AlertDialogFooter>
  </AlertDialogContent>
</AlertDialog>
