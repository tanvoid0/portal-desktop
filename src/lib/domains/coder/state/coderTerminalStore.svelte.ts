/**
 * Per-thread terminal tabs for Coder sessions.
 * Lightweight — no full TerminalWorkspace chrome; backs the session dock.
 */

export type CoderTerminalKind = "interactive" | "oneshot";
export type CoderTerminalOwner = "user" | "agent";

export interface CoderTerminalTab {
  id: string;
  label: string;
  tabId: string;
  kind: CoderTerminalKind;
  createdBy: CoderTerminalOwner;
  running: boolean;
  createdAt: string;
}

const STORAGE_KEY = "portal-coder-terminals-v1";

function storageKey(threadId: string): string {
  return `${STORAGE_KEY}:${threadId}`;
}

function loadTabs(threadId: string): CoderTerminalTab[] {
  if (typeof window === "undefined") return [];
  try {
    const raw = localStorage.getItem(storageKey(threadId));
    if (!raw) return [];
    return JSON.parse(raw) as CoderTerminalTab[];
  } catch {
    return [];
  }
}

function saveTabs(threadId: string, tabs: CoderTerminalTab[]) {
  if (typeof window === "undefined") return;
  try {
    localStorage.setItem(storageKey(threadId), JSON.stringify(tabs));
  } catch {
    /* ignore */
  }
}

export function coderTerminalTabId(threadId: string, terminalId: string): string {
  return `coder-${threadId}-${terminalId}`;
}

class CoderTerminalStoreState {
  /** threadId → tabs */
  private tabsByThread = $state<Record<string, CoderTerminalTab[]>>({});
  /** threadId → active terminal id */
  activeByThread = $state<Record<string, string>>({});
  revision = $state(0);

  private bump() {
    this.revision += 1;
  }

  /** Read-only — safe inside $derived / templates (never writes $state). */
  private peekTabs(threadId: string): CoderTerminalTab[] {
    this.revision;
    return this.tabsByThread[threadId] ?? loadTabs(threadId);
  }

  /** Populate in-memory cache from localStorage before a write. */
  private ensureLoaded(threadId: string) {
    if (!this.tabsByThread[threadId]) {
      this.tabsByThread[threadId] = loadTabs(threadId);
    }
  }

  tabsFor(threadId: string): CoderTerminalTab[] {
    return this.peekTabs(threadId);
  }

  getTab(threadId: string, terminalId: string): CoderTerminalTab | undefined {
    return this.peekTabs(threadId).find((t) => t.id === terminalId);
  }

  activeId(threadId: string): string | null {
    const tabs = this.peekTabs(threadId);
    const active = this.activeByThread[threadId];
    if (active && tabs.some((t) => t.id === active)) return active;
    return tabs[0]?.id ?? null;
  }

  setActive(threadId: string, terminalId: string) {
    if (this.activeByThread[threadId] === terminalId) return;
    this.activeByThread[threadId] = terminalId;
    this.bump();
  }

  createTab(
    threadId: string,
    opts: {
      workspaceRoot: string;
      createdBy?: CoderTerminalOwner;
      kind?: CoderTerminalKind;
      id?: string;
      label?: string;
    },
  ): CoderTerminalTab {
    this.ensureLoaded(threadId);
    const tabs = this.tabsByThread[threadId] ?? [];
    const id = opts.id ?? crypto.randomUUID().slice(0, 8);
    const n = tabs.length + 1;
    const tab: CoderTerminalTab = {
      id,
      label: opts.label ?? `Terminal ${n}`,
      tabId: coderTerminalTabId(threadId, id),
      kind: opts.kind ?? "interactive",
      createdBy: opts.createdBy ?? "user",
      running: false,
      createdAt: new Date().toISOString(),
    };
    this.tabsByThread[threadId] = [...tabs, tab];
    saveTabs(threadId, this.tabsByThread[threadId]);
    this.activeByThread[threadId] = id;
    this.bump();
    return tab;
  }

  /** First interactive tab, or create one. Call from $effect / handlers only. */
  ensureDefault(threadId: string, workspaceRoot: string): CoderTerminalTab {
    const tabs = this.peekTabs(threadId);
    const existing = tabs.find((t) => t.kind === "interactive");
    if (existing) return existing;
    return this.createTab(threadId, { workspaceRoot, createdBy: "user" });
  }

  setRunning(threadId: string, terminalId: string, running: boolean) {
    this.ensureLoaded(threadId);
    const tabs = this.tabsByThread[threadId];
    if (!tabs) return;
    const i = tabs.findIndex((t) => t.id === terminalId);
    if (i < 0) return;
    tabs[i] = { ...tabs[i], running };
    this.tabsByThread[threadId] = [...tabs];
    saveTabs(threadId, tabs);
    this.bump();
  }

  removeTab(threadId: string, terminalId: string) {
    this.ensureLoaded(threadId);
    const tabs = (this.tabsByThread[threadId] ?? []).filter(
      (t) => t.id !== terminalId,
    );
    this.tabsByThread[threadId] = tabs;
    saveTabs(threadId, tabs);
    if (this.activeByThread[threadId] === terminalId) {
      this.activeByThread[threadId] = tabs[0]?.id ?? "";
    }
    this.bump();
  }

  /** Summary for agent tool descriptions. */
  listForAgent(threadId: string): { id: string; label: string; kind: string }[] {
    return this.peekTabs(threadId).map((t) => ({
      id: t.id,
      label: t.label,
      kind: t.kind,
    }));
  }

  clearThread(threadId: string) {
    delete this.tabsByThread[threadId];
    delete this.activeByThread[threadId];
    if (typeof window !== "undefined") {
      localStorage.removeItem(storageKey(threadId));
    }
    this.bump();
  }
}

export const coderTerminalStore = new CoderTerminalStoreState();
