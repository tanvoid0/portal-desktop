<script lang="ts">
  import { onMount } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Badge } from '$lib/components/ui/badge';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import { Switch } from '$lib/components/ui/switch';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Alert, AlertDescription } from '$lib/components/ui/alert';
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import Loader2 from '~icons/fa6-solid/spinner';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Trash2 from '~icons/fa6-solid/trash';
  import { toast } from '$lib/utils/toast';
  import { formatBytes } from '$lib/domains/shared/utils';
  import {
    listPrograms,
    scanLeftovers,
    silentAvailable,
    uninstallProgram,
    type InstalledProgram,
    type Leftover,
  } from '../services/uninstallerService';

  let programs = $state<InstalledProgram[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let search = $state('');
  let showSystem = $state(false);

  let target = $state<InstalledProgram | null>(null);
  let leftovers = $state<Leftover[]>([]);
  let scanning = $state(false);
  let selected = $state<Set<string>>(new Set());
  let cleanup = $state(true);
  let silent = $state(false);
  let running = $state(false);

  const visible = $derived.by(() => {
    const q = search.trim().toLowerCase();
    return programs
      .filter((p) => showSystem || !p.systemComponent)
      .filter((p) => !q || `${p.name} ${p.publisher}`.toLowerCase().includes(q));
  });

  async function load() {
    loading = true;
    error = null;
    try {
      programs = await listPrograms();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function open(p: InstalledProgram) {
    target = p;
    leftovers = [];
    selected = new Set();
    silent = false;
    cleanup = true;
    scanning = true;
    try {
      leftovers = await scanLeftovers(p.id);
      selected = new Set(leftovers.map((l) => l.path));
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      scanning = false;
    }
  }

  function toggle(path: string, on: boolean) {
    const next = new Set(selected);
    if (on) next.add(path);
    else next.delete(path);
    selected = next;
  }

  async function run() {
    if (!target) return;
    running = true;
    try {
      const paths = cleanup ? [...selected] : [];
      const r = await uninstallProgram(target.id, silent, paths);
      if (r.removed) {
        const c = r.cleanup;
        toast.success(
          c ? `${target.name} uninstalled. Removed ${c.removed.length} leftover(s).` : r.message
        );
        if (c?.failed.length) toast.error(`${c.failed.length} leftover(s) could not be removed.`);
        target = null;
        await load();
      } else {
        toast.error(r.message);
      }
    } catch (e) {
      toast.error(e instanceof Error ? e.message : String(e));
    } finally {
      running = false;
    }
  }

  onMount(load);
</script>

<div class="space-y-4">
  <div class="flex flex-col gap-3 sm:flex-row sm:items-center">
    <Input bind:value={search} placeholder="Search programs or publishers…" class="sm:max-w-sm" />
    <div class="flex items-center gap-2">
      <Switch id="show-system" bind:checked={showSystem} />
      <Label for="show-system">Show system components</Label>
    </div>
    <Button variant="outline" size="sm" onclick={load} disabled={loading} class="sm:ml-auto">
      <RefreshCw class="mr-2 h-4 w-4" /> Refresh
    </Button>
  </div>

  {#if error}
    <Alert variant="destructive"><AlertDescription>{error}</AlertDescription></Alert>
  {:else if loading}
    <div class="flex items-center gap-2 text-muted-foreground">
      <Loader2 class="h-4 w-4 animate-spin" /> Reading installed programs…
    </div>
  {:else if visible.length === 0}
    <p class="text-muted-foreground">No programs match.</p>
  {:else}
    <div class="grid gap-2">
      {#each visible as p (p.id)}
        <Card>
          <CardContent class="flex items-center gap-3 p-3">
            <div class="min-w-0 flex-1">
              <div class="truncate font-medium">{p.name}</div>
              <div class="truncate text-sm text-muted-foreground">
                {[p.publisher, p.version, p.sizeKb ? formatBytes(p.sizeKb * 1024) : '']
                  .filter(Boolean)
                  .join(' · ')}
              </div>
            </div>
            <Button variant="outline" size="sm" onclick={() => open(p)}>
              <Trash2 class="mr-2 h-4 w-4" /> Uninstall
            </Button>
          </CardContent>
        </Card>
      {/each}
    </div>
  {/if}
</div>

<Dialog open={target !== null} onOpenChange={(o) => !o && !running && (target = null)}>
  <DialogContent class="max-w-[min(100vw-2rem,40rem)]">
    <DialogHeader>
      <DialogTitle>Uninstall {target?.name}</DialogTitle>
      <DialogDescription>
        Runs the program's own uninstaller first. Leftovers are found beforehand and removed only if
        it succeeds — folders go to the Recycle Bin, registry keys are backed up.
      </DialogDescription>
    </DialogHeader>

    {#if target}
      <div class="flex items-center gap-2">
        <Switch id="silent" bind:checked={silent} disabled={!silentAvailable(target)} />
        <Label for="silent">Silent (no dialogs)</Label>
        {#if !silentAvailable(target)}<Badge variant="secondary">not supported</Badge>{/if}
      </div>

      <div class="flex items-center gap-2">
        <Switch id="cleanup" bind:checked={cleanup} />
        <Label for="cleanup">Also remove leftovers</Label>
      </div>

      {#if cleanup}
        <div class="max-h-60 space-y-1 overflow-y-auto rounded-md border p-2">
          {#if scanning}
            <div class="flex items-center gap-2 text-sm text-muted-foreground">
              <Loader2 class="h-4 w-4 animate-spin" /> Scanning for leftovers…
            </div>
          {:else if leftovers.length === 0}
            <p class="text-sm text-muted-foreground">No leftovers found.</p>
          {:else}
            {#each leftovers as l (l.path)}
              <div class="flex items-start gap-2 text-sm">
                <Checkbox
                  checked={selected.has(l.path)}
                  onCheckedChange={(v) => toggle(l.path, v === true)}
                  class="mt-0.5"
                />
                <span class="min-w-0 flex-1 break-all">{l.path}</span>
                <Badge variant="outline">
                  {l.kind === 'folder' ? formatBytes(l.sizeBytes) : 'registry'}
                </Badge>
              </div>
            {/each}
          {/if}
        </div>
      {/if}
    {/if}

    <DialogFooter>
      <Button variant="outline" onclick={() => (target = null)} disabled={running}>Cancel</Button>
      <Button variant="destructive" onclick={run} disabled={running || scanning}>
        {#if running}<Loader2 class="mr-2 h-4 w-4 animate-spin" />{/if}
        Uninstall
      </Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
