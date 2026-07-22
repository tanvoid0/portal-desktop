/**
 * Unified command block store — OSC 133 events, readonly captures, AI blocks.
 */

import { writable, derived, get } from "svelte/store";
import type { TerminalCommand } from "../types";

export type CommandBlockSource = "pty" | "readonly" | "ai";

export interface CapturedCommand {
  id: string;
  tabId: string;
  processId?: string;
  command: string;
  output: string;
  exitCode?: number;
  duration?: number;
  workingDirectory?: string;
  timestamp: string;
  isExpanded?: boolean;
  source: CommandBlockSource;
  status?: "pending" | "running" | "completed" | "failed" | "paused";
}

interface CommandBlockState {
  blocksByTab: Record<string, CapturedCommand[]>;
  listenerStarted: boolean;
}

const initialState: CommandBlockState = {
  blocksByTab: {},
  listenerStarted: false,
};

/**
 * Cap on the output retained per command block.
 *
 * Output arrives one PTY chunk at a time and every chunk re-concatenates the
 * whole buffer here, then re-runs `stripForDisplay` over it in CommandBlock.
 * Uncapped that is O(total²): a `cargo build` emitting a few MB stalls the UI
 * for tens of seconds. The full stream is still in the terminal view itself —
 * a block only needs enough scrollback to be readable.
 *
 * ponytail: bounds the pathological case but leaves O(cap × chunks) of strip
 * work. If fast builds still feel choppy, coalesce chunks on a rAF before
 * writing rather than raising/lowering this number.
 */
export const MAX_BLOCK_OUTPUT = 128 * 1024;
export const TRUNCATION_NOTICE = "…[earlier output truncated]\n";
/** How far past the cut point to look for a line boundary before hard-cutting. */
const LINE_BOUNDARY_SCAN = 4096;

/** Append a chunk, keeping only the trailing `MAX_BLOCK_OUTPUT` characters. */
export function appendCapped(existing: string, content: string): string {
  const combined = existing + content;
  if (combined.length <= MAX_BLOCK_OUTPUT) return combined;

  const excess = combined.length - MAX_BLOCK_OUTPUT;
  // Prefer cutting at a line boundary so we don't slice an escape sequence in
  // half — but only if one is close by. Scanning to an arbitrarily distant
  // newline would discard live output (or everything, for output that has no
  // newlines at all), so fall back to a hard cut.
  const boundary = combined.indexOf("\n", excess);
  const kept =
    boundary !== -1 && boundary - excess <= LINE_BOUNDARY_SCAN
      ? combined.slice(boundary + 1)
      : combined.slice(excess);
  return TRUNCATION_NOTICE + kept;
}

function parseDateMaybe(v: unknown): string {
  if (!v) return new Date().toISOString();
  const d = v instanceof Date ? v : new Date(String(v));
  if (Number.isNaN(d.getTime())) return new Date().toISOString();
  return d.toISOString();
}

function toNumberMaybe(v: unknown): number | undefined {
  if (v === null || v === undefined) return undefined;
  const n = Number(v);
  return Number.isFinite(n) ? n : undefined;
}

function normalizeShellIntegrationEvent(raw: unknown): {
  type: string;
  payload: Record<string, unknown>;
  process_id?: string;
} | null {
  // Unit enum variants (e.g. PromptDetected) serialize as bare strings.
  if (typeof raw === "string") return { type: raw, payload: {} };
  if (!raw || typeof raw !== "object") return null;
  const obj = raw as Record<string, unknown>;

  if ("process_id" in obj && "event" in obj) {
    const inner = normalizeShellIntegrationEvent(obj.event);
    if (!inner) return null;
    return {
      type: inner.type,
      payload: inner.payload,
      process_id: String(obj.process_id),
    };
  }

  if (typeof obj.type === "string" && "payload" in obj) {
    return {
      type: obj.type,
      payload: (obj.payload as Record<string, unknown>) ?? {},
    };
  }

  const keys = Object.keys(obj);
  if (keys.length === 1) {
    const type = keys[0];
    return { type, payload: (obj[type] as Record<string, unknown>) ?? {} };
  }

  return null;
}

function createCommandBlockStore() {
  const { subscribe, set, update } = writable<CommandBlockState>(initialState);
  const tabSubscribers = new Map<
    string,
    Set<(blocks: CapturedCommand[]) => void>
  >();
  const processToTab = new Map<string, string>();
  /** Processes whose shell emitted at least one OSC 133 event — for these,
   *  blocks are created by shell integration, not manually on submit. */
  const integrationActive = new Set<string>();
  let unsubs: (() => void)[] = [];

  function notifyTab(tabId: string, blocks: CapturedCommand[]) {
    tabSubscribers.get(tabId)?.forEach((cb) => cb(blocks));
  }

  function getBlocksForTab(tabId: string): CapturedCommand[] {
    return get({ subscribe }).blocksByTab[tabId] ?? [];
  }

  function upsertBlock(tabId: string, block: CapturedCommand) {
    update((state) => {
      const existing = state.blocksByTab[tabId] ?? [];
      const idx = existing.findIndex((b) => b.id === block.id);
      const next =
        idx === -1
          ? [block, ...existing]
          : existing.map((b, i) => (i === idx ? block : b));
      const blocksByTab = { ...state.blocksByTab, [tabId]: next };
      notifyTab(tabId, next);
      return { ...state, blocksByTab };
    });
  }

  function resolveTabId(processId?: string): string {
    if (!processId) return "global";
    return processToTab.get(processId) ?? processId;
  }

  async function startShellIntegrationListener() {
    const state = get({ subscribe });
    if (state.listenerStarted) return;

    try {
      const { listen } = await import("@tauri-apps/api/event");

      const onEvent = (event: { payload: unknown }) => {
        const normalized = normalizeShellIntegrationEvent(event.payload);
        if (!normalized) return;

        const { type, payload } = normalized;
        const scopedIdPrefix = normalized.process_id
          ? `${normalized.process_id}:`
          : "";
        const tabId = resolveTabId(normalized.process_id);
        if (normalized.process_id) {
          integrationActive.add(normalized.process_id);
        }

        if (type === "CommandDetected") {
          // Payload is the raw command string (unit enum variant).
          const command =
            typeof payload === "string" ? payload : String(payload ?? "");
          if (!command) return;
          update((state) => {
            const existing = state.blocksByTab[tabId] ?? [];
            const idx = existing.findIndex(
              (b) => b.status === "running" && b.source === "pty",
            );
            if (idx === -1) return state;
            const next = existing.map((b, i) =>
              i === idx ? { ...b, command } : b,
            );
            notifyTab(tabId, next);
            return {
              ...state,
              blocksByTab: { ...state.blocksByTab, [tabId]: next },
            };
          });
          return;
        }

        if (type === "CommandStarted" || type === "CommandStart") {
          const started = payload;
          if (!started?.id) return;
          // A manual block may exist for this same command (submitted before
          // we knew integration was live) — the shell-integration block
          // supersedes it.
          update((state) => {
            const existing = state.blocksByTab[tabId] ?? [];
            const next = existing.filter(
              (b) =>
                !(
                  b.status === "running" &&
                  b.source === "pty" &&
                  b.processId === normalized.process_id &&
                  !b.id.startsWith(scopedIdPrefix)
                ),
            );
            if (next.length === existing.length) return state;
            notifyTab(tabId, next);
            return {
              ...state,
              blocksByTab: { ...state.blocksByTab, [tabId]: next },
            };
          });
          upsertBlock(tabId, {
            id: `${scopedIdPrefix}${String(started.id)}`,
            tabId,
            processId: normalized.process_id,
            command: String(started.command ?? ""),
            output: "",
            exitCode: undefined,
            duration: undefined,
            workingDirectory: started.working_directory
              ? String(started.working_directory)
              : started.workingDirectory
                ? String(started.workingDirectory)
                : undefined,
            timestamp: parseDateMaybe(started.start_time ?? started.timestamp),
            isExpanded: true,
            source: "pty",
            status: "running",
          });
        } else if (
          type === "CommandCompleted" ||
          type === "CommandEnd" ||
          type === "CommandEndEvent"
        ) {
          const completed = payload;
          if (!completed?.id) return;
          const exitCode = toNumberMaybe(
            completed.exit_code ?? completed.exitCode,
          );
          upsertBlock(tabId, {
            id: `${scopedIdPrefix}${String(completed.id)}`,
            tabId,
            processId: normalized.process_id,
            command: String(completed.command ?? ""),
            output: String(completed.output ?? ""),
            exitCode,
            duration: toNumberMaybe(
              completed.duration ?? completed.durationMs,
            ),
            workingDirectory: completed.working_directory
              ? String(completed.working_directory)
              : completed.workingDirectory
                ? String(completed.workingDirectory)
                : undefined,
            timestamp: parseDateMaybe(
              completed.start_time ?? completed.timestamp,
            ),
            isExpanded: true,
            source: "pty",
            status:
              exitCode === 0 || exitCode === undefined ? "completed" : "failed",
          });
        }
      };

      unsubs = [
        (await listen("shell-integration-event-v2", onEvent)) as () => void,
        (await listen("shell-integration-event", onEvent)) as () => void,
      ];
      update((s) => ({ ...s, listenerStarted: true }));
    } catch {
      // Browser / non-Tauri — blocks stay empty until manual capture.
    }
  }

  return {
    subscribe,

    startShellIntegrationListener,

    getBlocksForTab,

    getLatestBlock(tabId: string): CapturedCommand | null {
      const blocks = getBlocksForTab(tabId);
      return blocks[0] ?? null;
    },

    subscribeToBlocks(
      tabId: string,
      callback: (blocks: CapturedCommand[]) => void,
    ): () => void {
      if (!tabSubscribers.has(tabId)) {
        tabSubscribers.set(tabId, new Set());
      }
      tabSubscribers.get(tabId)!.add(callback);
      callback(getBlocksForTab(tabId));
      return () => tabSubscribers.get(tabId)?.delete(callback);
    },

    addBlock(
      tabId: string,
      block: Omit<CapturedCommand, "id" | "tabId" | "timestamp"> & {
        id?: string;
        timestamp?: string;
      },
    ): string {
      const id = block.id ?? crypto.randomUUID();
      upsertBlock(tabId, {
        id,
        tabId,
        command: block.command,
        output: block.output,
        exitCode: block.exitCode,
        duration: block.duration,
        workingDirectory: block.workingDirectory,
        processId: block.processId,
        timestamp: block.timestamp ?? new Date().toISOString(),
        isExpanded: block.isExpanded ?? true,
        source: block.source,
        status: block.status ?? "running",
      });
      return id;
    },

    /** True once the process's shell has emitted OSC 133 events — blocks are
     *  then created/completed by shell integration, not manual submits. */
    hasIntegration(processId: string): boolean {
      return integrationActive.has(processId);
    },

    /** Stream a raw PTY output chunk into the currently running block for
     *  this process (live output while the command runs). No-op when no
     *  block is running. Raw chunk may contain ANSI; strip at render. */
    appendToRunningBlock(processId: string, content: string) {
      const tabId = resolveTabId(processId);
      update((state) => {
        const existing = state.blocksByTab[tabId] ?? [];
        const idx = existing.findIndex(
          (b) => b.status === "running" && b.source === "pty",
        );
        if (idx === -1) return state;
        const next = existing.map((b, i) =>
          i === idx ? { ...b, output: appendCapped(b.output, content) } : b,
        );
        notifyTab(tabId, next);
        return {
          ...state,
          blocksByTab: { ...state.blocksByTab, [tabId]: next },
        };
      });
    },

    appendOutput(tabId: string, blockId: string, content: string) {
      update((state) => {
        const existing = state.blocksByTab[tabId] ?? [];
        const next = existing.map((b) =>
          b.id === blockId ? { ...b, output: appendCapped(b.output, content) } : b,
        );
        notifyTab(tabId, next);
        return { ...state, blocksByTab: { ...state.blocksByTab, [tabId]: next } };
      });
    },

    completeBlock(tabId: string, blockId: string, exitCode?: number) {
      update((state) => {
        const existing = state.blocksByTab[tabId] ?? [];
        const next = existing.map((b) =>
          b.id === blockId
            ? {
                ...b,
                exitCode,
                status:
                  exitCode === 0 || exitCode === undefined
                    ? ("completed" as const)
                    : ("failed" as const),
              }
            : b,
        );
        notifyTab(tabId, next);
        return { ...state, blocksByTab: { ...state.blocksByTab, [tabId]: next } };
      });
    },

    captureReadonlyResult(
      tabId: string,
      result: TerminalCommand | { command: string; output?: string; exitCode?: number },
    ): string {
      const output = "output" in result ? (result.output ?? "") : "";
      const exitCode =
        "exitCode" in result && result.exitCode !== undefined
          ? result.exitCode
          : "status" in result && result.status === "failed"
            ? 1
            : 0;
      return this.addBlock(tabId, {
        command: result.command,
        output,
        exitCode,
        source: "readonly",
        status: exitCode === 0 ? "completed" : "failed",
      });
    },

    registerProcessTab(processId: string, tabId: string) {
      processToTab.set(processId, tabId);
      update((state) => {
        const processBlocks = state.blocksByTab[processId];
        if (!processBlocks?.length) return state;
        const tabBlocks = state.blocksByTab[tabId] ?? [];
        const merged = [
          ...processBlocks.map((b) => ({ ...b, tabId })),
          ...tabBlocks,
        ];
        const blocksByTab = { ...state.blocksByTab, [tabId]: merged };
        delete blocksByTab[processId];
        notifyTab(tabId, merged);
        return { ...state, blocksByTab };
      });
    },

    clearBlocks(tabId: string) {
      update((state) => {
        const blocksByTab = { ...state.blocksByTab, [tabId]: [] };
        notifyTab(tabId, []);
        return { ...state, blocksByTab };
      });
    },

    reset() {
      for (const unsub of unsubs) {
        try {
          unsub();
        } catch {
          // ignore
        }
      }
      unsubs = [];
      integrationActive.clear();
      processToTab.clear();
      set(initialState);
    },
  };
}

export const commandBlockStore = createCommandBlockStore();

export const blocksForActiveTab = (tabId: string) =>
  derived({ subscribe: commandBlockStore.subscribe }, () =>
    commandBlockStore.getBlocksForTab(tabId),
  );
