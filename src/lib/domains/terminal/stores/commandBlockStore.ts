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

        if (type === "CommandStarted" || type === "CommandStart") {
          const started = payload;
          if (!started?.id) return;
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

    appendOutput(tabId: string, blockId: string, content: string) {
      update((state) => {
        const existing = state.blocksByTab[tabId] ?? [];
        const next = existing.map((b) =>
          b.id === blockId ? { ...b, output: b.output + content } : b,
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
      set(initialState);
    },
  };
}

export const commandBlockStore = createCommandBlockStore();

export const blocksForActiveTab = (tabId: string) =>
  derived({ subscribe: commandBlockStore.subscribe }, () =>
    commandBlockStore.getBlocksForTab(tabId),
  );
