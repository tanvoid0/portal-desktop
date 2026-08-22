<!--
	Disk Utility shell.

	Five tabs, named for what you get rather than for the code behind them, and
	each one says in a line what its scan looks at. The old split had seven,
	including two ("Projects" and "Dev tools") that ran the same Rust cleaner and
	one ("Settings") that held a single paragraph.
-->

<script lang="ts">
  import DashboardTab from './DashboardTab.svelte';
  import CleanupTab from './CleanupTab.svelte';
  import DevToolsTab from './DevToolsTab.svelte';
  import HistoryTab from './HistoryTab.svelte';
  import RulesTab from './RulesTab.svelte';
  import { Tabs, TabsList, TabsTrigger } from '$lib/components/ui/tabs';
  import { HardDrive } from '@lucide/svelte';
  import { onMount } from 'svelte';

  type Tab = 'overview' | 'files' | 'developer' | 'history' | 'rules';

  const TABS: { id: Tab; label: string; blurb: string }[] = [
    {
      id: 'overview',
      label: 'Overview',
      blurb: 'How full each drive is, how much you have reclaimed, and what was moved recently.',
    },
    {
      id: 'files',
      label: 'Files',
      blurb:
        'Scan any folder for junk you can spare — caches, logs, installers, old downloads. Each proposal is rated Safe, Review or Danger.',
    },
    {
      id: 'developer',
      label: 'Developer',
      blurb:
        'Build output and dependency folders (node_modules, target, dist) plus Docker and Podman leftovers. Your toolchain rebuilds all of it.',
    },
    {
      id: 'history',
      label: 'History',
      blurb: 'Everything this app has moved, newest first. Undo by restoring from the Recycle Bin.',
    },
    {
      id: 'rules',
      label: 'Rules',
      blurb: 'Folders no scan may ever touch, and where the optional AI review gets its provider.',
    },
  ];

  let tab = $state<Tab>('overview');
  const blurb = $derived(TABS.find((t) => t.id === tab)?.blurb ?? '');

  // Deep-link request from the dashboard: switch tab + optionally auto-scan a path.
  // `pending.tab` keeps the child-facing names the scan tabs already match on.
  let pending = $state<{ tab: string; path?: string; seq: number } | null>(null);
  let seq = 0;

  function go(target: 'cleanup' | 'projects' | 'devtools', path?: string) {
    // `projects` folded into the developer scan; keep the old name working so
    // links from the project audit (and any bookmark) still land somewhere real.
    const child = target === 'projects' ? 'devtools' : target;
    tab = child === 'cleanup' ? 'files' : 'developer';
    seq += 1;
    pending = { tab: child, path, seq };
  }

  /** Tab ids that used to exist, mapped onto the ones that replaced them. */
  const LEGACY: Record<string, Tab> = {
    dashboard: 'overview',
    cleanup: 'files',
    projects: 'developer',
    devtools: 'developer',
    protected: 'rules',
    settings: 'rules',
  };

  onMount(() => {
    const params = new URLSearchParams(window.location.search);
    const raw = params.get('tab');
    if (!raw) return;
    const target = TABS.some((t) => t.id === raw) ? (raw as Tab) : LEGACY[raw];
    if (!target) return;
    if (target === 'files' || target === 'developer') {
      go(target === 'files' ? 'cleanup' : 'devtools', params.get('path') ?? undefined);
    } else {
      tab = target;
    }
  });
</script>

<div class="w-full">
  <header class="mb-6">
    <div class="mb-2 flex items-center gap-2.5">
      <div
        class="flex h-6 w-6 items-center justify-center rounded-md bg-primary text-primary-foreground"
      >
        <HardDrive class="size-3.5" />
      </div>
      <h1 class="text-lg font-semibold tracking-tight text-foreground">Disk Utility</h1>
    </div>
    <p class="max-w-2xl text-sm text-muted-foreground">
      Scans propose, you decide. Nothing goes without your tick, and what goes lands in the Recycle
      Bin — except container resources, which Docker and Podman remove for good.
    </p>
  </header>

  <Tabs value={tab} onValueChange={(v) => (tab = v as Tab)} class="mb-3">
    <TabsList>
      {#each TABS as t (t.id)}
        <TabsTrigger value={t.id}>{t.label}</TabsTrigger>
      {/each}
    </TabsList>
  </Tabs>

  <p class="mb-6 max-w-3xl text-sm text-muted-foreground">{blurb}</p>

  <div class={tab === 'overview' ? '' : 'hidden'}>
    <DashboardTab active={tab === 'overview'} {go} />
  </div>
  <div class={tab === 'files' ? '' : 'hidden'}>
    <CleanupTab {pending} />
  </div>
  <div class={tab === 'developer' ? '' : 'hidden'}>
    <DevToolsTab {pending} />
  </div>
  <div class={tab === 'history' ? '' : 'hidden'}>
    <HistoryTab active={tab === 'history'} />
  </div>
  <div class={tab === 'rules' ? '' : 'hidden'}>
    <RulesTab active={tab === 'rules'} />
  </div>
</div>
