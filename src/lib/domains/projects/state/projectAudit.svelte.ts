/**
 * State, derivations and handlers for the Project Audit view. Construct it
 * during component init — it registers onMount/onDestroy and an $effect.
 */
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { save as saveDialog } from '@tauri-apps/plugin-dialog';
import { onDestroy, onMount } from 'svelte';
import { goto } from '$app/navigation';
import { logger } from '$lib/domains/shared/services/logger';
import { confirmAction } from '$lib/utils/confirm';
import { invokeClient } from '$lib/utils/invokeClient';
// Per-row actions fail while the reader is deep in the list, where the scan
// card's inline error is scrolled off. Only the scan reports inline.
import { toast } from '$lib/utils/toast';
// The audit reports reclaimable bytes; the Disk utility deletes them. Sharing
// its formatter keeps one number reading the same on both pages.
import { fmtBytes } from '$lib/domains/disk/utils';
import type { PageStat } from '$lib/components/shell';
import {
  defaultDiscoveryConfig,
  discoverAndImport,
  DISCOVERY_PROGRESS_EVENT,
  exportDiscoveryCsv,
  getDiscoveryState,
  saveDiscoveryConfig,
  setProjectTriage,
  setProjectTriageBulk,
  type TriageDecision,
} from '$lib/domains/projects/api/discoveryApi';
import { toCsv } from '$lib/domains/projects/utils/auditCsv';
import { DECISIONS } from '$lib/domains/projects/utils/auditFormat';
import type { DiscoveryProgress } from '$lib/domains/projects/generated/DiscoveryProgress';
import type { DiscoveredProject } from '$lib/domains/projects/generated/DiscoveredProject';
import type { DiscoveryReport } from '$lib/domains/projects/generated/DiscoveryReport';
import type { DiscoveryChange } from '$lib/domains/projects/generated/DiscoveryChange';

const log = logger.createScoped('ProjectAudit');

export type AuditTab = 'triage' | 'risk' | 'duplicates' | 'tree' | 'changes';

/** `/` and `.` are legal in a logical id but not in a DOM id. */
export function auditRowId(path: string): string {
  return `audit-${path.replace(/[^a-zA-Z0-9]/g, '-')}`;
}

export class ProjectAuditState {
  roots = $state<{ name: string; path: string }[]>([]);
  newRootPath = $state('');
  report = $state<DiscoveryReport | null>(null);
  changes = $state<DiscoveryChange[]>([]);
  measureSizes = $state(false);
  markMissing = $state(false);
  scanning = $state(false);
  progress = $state<DiscoveryProgress | null>(null);
  error = $state<string | null>(null);
  tab = $state<AuditTab>('triage');
  search = $state('');
  sortBySize = $state(false);
  selected = $state<DiscoveredProject | null>(null);
  decisions = $state<Record<string, TriageDecision | null>>({});
  idByPath = $state<Record<string, number>>({});
  importSummary = $state<string | null>(null);
  exportedTo = $state<string | null>(null);
  /** Saved roots arrive async; without this the card claims "no folders yet" first. */
  loadingState = $state(true);
  /** Keyboard position in `filtered`. */
  cursor = $state(0);
  /** Paths ticked for a bulk decision. */
  marked = $state<Set<string>>(new Set());
  /**
   * The rail holds the scan config, the cursor row's facts and the bulk
   * actions. Below `md` ResponsivePanel renders it as a Sheet, so it needs an
   * open flag and a trigger; on desktop it is always on screen and this is
   * ignored.
   */
  railOpen = $state(false);
  /**
   * Roots and scan options are set up once and then touched monthly, so the
   * form stays folded behind the Scan button. Opened on mount when there is
   * nothing to scan — folding an empty form would hide the only way out.
   */
  scanOpen = $state(false);

  projects = $derived(this.report?.projects ?? []);

  filtered = $derived.by(() => {
    const q = this.search.trim().toLowerCase();
    let list = this.projects;
    if (this.tab === 'risk')
      list = list.filter((p) => p.riskLevel === 'high' || p.riskLevel === 'med');
    if (this.tab === 'duplicates') list = list.filter((p) => p.duplicateOf.length > 0);
    if (q) {
      list = list.filter((p) =>
        `${p.path} ${p.name} ${p.langs.join(' ')} ${p.description ?? ''}`.toLowerCase().includes(q)
      );
    }
    // Backend order is newest-first; sorting by size answers the other question.
    return this.sortBySize
      ? [...list].sort((a, b) => b.regenerableBytes - a.regenerableBytes)
      : list;
  });

  stats = $derived(this.report?.stats ?? null);
  undecided = $derived(this.projects.filter((p) => !this.decisions[p.path]).length);
  highRisk = $derived(this.projects.filter((p) => p.riskLevel === 'high'));

  /**
   * A cluster whose members no longer resolve to a scanned project draws
   * nothing. Dropping those here keeps the tab's empty state honest and stops
   * the "Duplicate sets" stat counting rows the tab will not show.
   */
  duplicateSets = $derived(
    (this.report?.clusters ?? [])
      .map((cluster) => ({
        cluster,
        members: cluster.members
          .map((m) => this.projects.find((p) => p.path === m))
          .filter((p): p is DiscoveredProject => !!p)
          .sort((a, b) => b.activity - a.activity),
      }))
      .filter((set) => set.members.length > 1)
  );

  /**
   * What the rail describes. On the keyboard tabs that is the `j`/`k` cursor
   * row, so the facts follow the loop without a click; the other tabs have no
   * cursor and point `selected` with their own inspect buttons.
   */
  focused = $derived(
    this.tab === 'triage' || this.tab === 'risk'
      ? (this.filtered[this.cursor] ?? null)
      : this.selected
  );

  /** `duplicateOf` holds cluster keys, not paths — resolve them to siblings. */
  duplicateSiblings = $derived.by(() => {
    const p = this.focused;
    if (!p?.duplicateOf.length) return [];
    const paths = new Set<string>();
    for (const set of this.duplicateSets) {
      if (!set.members.some((m) => m.path === p.path)) continue;
      for (const m of set.members) if (m.path !== p.path) paths.add(m.path);
    }
    return [...paths];
  });

  /** The six headline numbers as one strip instead of six cards (UX-002). */
  statStrip = $derived.by((): PageStat[] => {
    const stats = this.stats;
    if (!stats) return [];
    return [
      { label: 'Projects', value: stats.projects, description: `${stats.roots} roots` },
      {
        label: 'Active',
        value: stats.active,
        description: `${stats.recent} recent · ${stats.dormant} dormant · ${stats.stale} stale`,
      },
      {
        label: 'At risk',
        value: this.highRisk.length,
        description: `${stats.unpushed} unpushed commits`,
      },
      {
        label: 'Duplicates',
        value: this.duplicateSets.length,
        description: 'sets sharing a git remote or a folder name',
      },
      { label: 'Undecided', value: this.undecided, description: `of ${stats.projects}` },
      {
        label: 'Reclaimable',
        value: stats.regenerableBytes < 0 ? '—' : fmtBytes(stats.regenerableBytes),
        description: stats.regenerableBytes < 0 ? 'not measured' : 'build output',
      },
    ];
  });

  constructor() {
    let unlisten: UnlistenFn | null = null;
    listen<DiscoveryProgress>(DISCOVERY_PROGRESS_EVENT, (e) => {
      this.progress = e.payload;
    })
      .then((fn) => (unlisten = fn))
      .catch((e) => log.warn('progress listener failed', e));

    onDestroy(() => unlisten?.());

    // The last scan is on disk, so the view opens populated instead of demanding
    // a fresh walk of every drive before it can show anything.
    onMount(async () => {
      try {
        const state = await getDiscoveryState();
        this.roots = state.config.roots ?? [];
        this.scanOpen = this.roots.length === 0;
        this.measureSizes = state.config.measureSizes;
        this.markMissing = state.config.markMissing;
        this.report = state.report;
        this.changes = state.changes;
        if (state.report) await this.loadRegistryIds();
      } catch (e) {
        log.warn('could not load saved audit state', e);
      } finally {
        this.loadingState = false;
      }
    });

    // A filter change can leave the cursor past the end of the list.
    $effect(() => {
      if (this.cursor > this.filtered.length - 1)
        this.cursor = Math.max(0, this.filtered.length - 1);
    });
  }

  /** Roots and scan options outlive the session; the report is written by scans. */
  persistConfig = async () => {
    try {
      await saveDiscoveryConfig(this.currentConfig());
    } catch (e) {
      log.warn('could not save audit config', e);
    }
  };

  currentConfig() {
    return {
      ...defaultDiscoveryConfig(this.roots),
      measureSizes: this.measureSizes,
      markMissing: this.markMissing,
    };
  }

  /** Uses the same native picker the create-project flow uses. */
  addRoot = async () => {
    try {
      const picked =
        this.newRootPath.trim() || (await invokeClient.post<string | null>('select_directory'));
      if (!picked) return;
      const base =
        picked
          .replace(/[\\/]+$/, '')
          .split(/[\\/]/)
          .pop() ?? 'root';
      if (this.roots.some((r) => r.path === picked)) return;
      // The name prefixes every project id from this root, and archiving
      // missing projects trusts that prefix to say which root a row came from.
      // Two folders called `production` on different drives would make one
      // root's rows look like the other's, so names are kept unique.
      let name = base;
      for (let n = 2; this.roots.some((r) => r.name === name); n++) name = `${base}-${n}`;
      this.roots = [...this.roots, { name, path: picked }];
      this.newRootPath = '';
      await this.persistConfig();
    } catch (e) {
      toast.error('Could not add that folder', e);
    }
  };

  removeRoot = async (path: string) => {
    this.roots = this.roots.filter((r) => r.path !== path);
    await this.persistConfig();
  };

  scan = async () => {
    if (!this.roots.length) {
      this.error = 'Add at least one folder to scan.';
      return;
    }
    this.scanning = true;
    this.error = null;
    this.importSummary = null;
    this.exportedTo = null;
    this.progress = null;
    // A selection made against the previous result set would silently apply to
    // whatever those paths mean now, or drop out of the batch without saying so.
    this.marked = new Set();
    this.cursor = 0;
    try {
      const result = await discoverAndImport(this.currentConfig(), this.markMissing);
      this.report = result.report;
      this.changes = [...result.changes, ...this.changes];
      this.importSummary =
        `${result.inserted.length} added · ${result.updated.length} refreshed · ` +
        `${result.adopted.length} linked to existing rows` +
        (result.missing.length ? ` · ${result.missing.length} archived as missing` : '') +
        (result.changes.length ? ` · ${result.changes.length} changes since last scan` : '');
      await this.loadRegistryIds();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      this.error = msg === 'cancelled' ? 'Scan cancelled — nothing was imported.' : msg;
    } finally {
      this.scanning = false;
      this.progress = null;
    }
  };

  /** Triage writes by registry id, so map discovery paths back to rows. */
  async loadRegistryIds() {
    try {
      const rows =
        await invokeClient.post<
          { id: number; discovery_path?: string | null; triage_decision?: string | null }[]
        >('get_all_projects');
      const ids: Record<string, number> = {};
      const dec: Record<string, TriageDecision | null> = {};
      for (const r of rows ?? []) {
        const key = r.discovery_path ?? null;
        if (!key) continue;
        ids[key] = r.id;
        dec[key] = (r.triage_decision as TriageDecision | null) ?? null;
      }
      this.idByPath = ids;
      this.decisions = dec;
    } catch (e) {
      log.warn('could not map registry ids', e);
    }
  }

  decide = async (p: DiscoveredProject, decision: TriageDecision) => {
    const id = this.idByPath[p.path];
    if (!id) {
      toast.error('Not in the registry yet', `Run a scan before triaging ${p.name}.`);
      return;
    }
    const next = this.decisions[p.path] === decision ? null : decision;
    if (next === 'delete') {
      const ok = await confirmAction(
        'This only records the decision — nothing is removed from disk. Use the Disk tab to reclaim space.',
        `Mark ${p.name} for deletion?`
      );
      if (!ok) return;
    }
    this.decisions = { ...this.decisions, [p.path]: next };
    try {
      await setProjectTriage(id, next);
    } catch (e) {
      this.decisions = { ...this.decisions, [p.path]: this.decisions[p.path] ?? null };
      toast.error(`Could not save the decision for ${p.name}`, e);
    }
  };

  /** Applies one decision to every ticked project in a single round trip. */
  decideMarked = async (decision: TriageDecision) => {
    const paths = [...this.marked].filter((p) => this.idByPath[p]);
    if (!paths.length) {
      toast.error(
        'Nothing to apply',
        'None of the selected projects are in the registry yet — run a scan first.'
      );
      return;
    }
    if (decision === 'delete') {
      const ok = await confirmAction(
        'This only records the decision — nothing is removed from disk.',
        `Mark ${paths.length} projects for deletion?`
      );
      if (!ok) return;
    }
    try {
      await setProjectTriageBulk(
        paths.map((p) => this.idByPath[p]),
        decision
      );
      const next = { ...this.decisions };
      for (const p of paths) next[p] = decision;
      this.decisions = next;
      this.marked = new Set();
      // The selection clearing is the only other sign the bulk write landed.
      toast.success(`${paths.length} marked ${decision}`);
    } catch (e) {
      toast.error(`Could not mark ${paths.length} projects ${decision}`, e);
    }
  };

  toggleMark = (path: string) => {
    const next = new Set(this.marked);
    if (!next.delete(path)) next.add(path);
    this.marked = next;
  };

  clearMarked = () => {
    this.marked = new Set();
  };

  /**
   * The triage loop: `j`/`k` to move, `1`–`4` to decide, `o` to open, `x` to
   * tick for a bulk decision. Only on the card tabs — the tree and the change
   * log have no cursor.
   */
  onKeydown = (e: KeyboardEvent) => {
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    const el = e.target as HTMLElement | null;
    if (el && (el.isContentEditable || /^(INPUT|TEXTAREA|SELECT)$/.test(el.tagName))) return;
    if (!this.report || (this.tab !== 'triage' && this.tab !== 'risk')) return;
    const current = this.filtered[this.cursor];

    switch (e.key) {
      case 'j':
      case 'ArrowDown':
        this.move(1);
        break;
      case 'k':
      case 'ArrowUp':
        this.move(-1);
        break;
      case '1':
      case '2':
      case '3':
      case '4':
        if (current) void this.decide(current, DECISIONS[Number(e.key) - 1]);
        break;
      case 'o':
        if (current) void this.openFolder(current.abs);
        break;
      case 'x':
        if (current) this.toggleMark(current.path);
        break;
      case 'Escape':
        this.marked = new Set();
        this.selected = null;
        break;
      default:
        return;
    }
    e.preventDefault();
  };

  move(delta: number) {
    if (!this.filtered.length) return;
    this.cursor = Math.min(this.filtered.length - 1, Math.max(0, this.cursor + delta));
    this.selected = this.filtered[this.cursor];
    // Moving real focus, not just the highlight: a screen reader follows the
    // cursor, and Tab from here resumes inside the row the reader is looking at.
    const row = document.getElementById(auditRowId(this.filtered[this.cursor].path));
    row?.focus({ preventScroll: true });
    row?.scrollIntoView({ block: 'nearest' });
  }

  exportCsv = async () => {
    try {
      const path = await saveDialog({
        defaultPath: 'project-audit.csv',
        filters: [{ name: 'CSV', extensions: ['csv'] }],
      });
      if (!path) return;
      this.exportedTo = await exportDiscoveryCsv(path, toCsv(this.filtered, this.decisions));
    } catch (e) {
      toast.error('Export failed', e);
    }
  };

  /**
   * Hands a project to the Disk utility, which owns measuring and deleting.
   * This view records what a project *is*; it never removes bytes itself.
   */
  cleanInDisk(p: DiscoveredProject) {
    void goto(`/utilities/disk?tab=developer&path=${encodeURIComponent(p.abs)}`);
  }

  openFolder = async (path: string) => {
    try {
      await invokeClient.post('open_project_in_explorer', { path });
    } catch (e) {
      toast.error('Could not open that folder', e);
    }
  };
}
