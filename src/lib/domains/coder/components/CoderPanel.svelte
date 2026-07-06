<script lang="ts">
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Badge } from "$lib/components/ui/badge";
  import { Bot, Send, User, Settings2, Trash2, Plus, MessageSquare } from "@lucide/svelte";
  import { coderService } from "../services/coderService.js";
  import type {
    ChatMessage,
    CoderRunResult,
    CoderThread,
    PendingApproval,
    PermissionMode,
    PermissionRule,
    ToolCall,
  } from "../types.js";
  import ToolCallCard from "./ToolCallCard.svelte";
  import ApprovalBanner from "./ApprovalBanner.svelte";

  let workspaceRoot = $state("");
  let thread = $state<CoderThread | null>(null);
  let threads = $state<CoderThread[]>([]);
  let messages = $state<ChatMessage[]>([]);
  let pending = $state<PendingApproval | null>(null);
  let input = $state("");
  let busy = $state(false);
  let running = $state(false);
  let streamingText = $state("");
  let error = $state<string | null>(null);

  let mode = $state<PermissionMode>("review");
  let rules = $state<PermissionRule[]>([]);
  let showSettings = $state(false);

  const MODES: { value: PermissionMode; label: string; hint: string }[] = [
    { value: "review", label: "Review", hint: "prompt on each mutating action" },
    { value: "auto-accept-all", label: "Auto-accept", hint: "run everything" },
    { value: "plan", label: "Plan", hint: "read-only, no writes/commands" },
  ];

  onMount(() => {
    const unlistens: Array<() => void> = [];

    (async () => {
      mode = await coderService.getMode();
      rules = await coderService.listRules();
      threads = await coderService.listThreads();

      unlistens.push(
        await coderService.onToken(({ thread_id, delta }) => {
          if (thread && thread_id === thread.id) streamingText += delta;
        }),
        await coderService.onMessage(({ thread_id, message }) => {
          if (thread && thread_id === thread.id) {
            // Finalize any in-progress streamed text as this real message lands.
            if (message.role === "assistant") streamingText = "";
            messages = [...messages, message];
          }
        }),
        await coderService.onPending(({ thread_id, pending: p }) => {
          if (thread && thread_id === thread.id) pending = p;
        }),
        await coderService.onDone(({ thread_id, exhausted }) => {
          if (thread && thread_id === thread.id) {
            running = false;
            streamingText = "";
            if (exhausted) error = "Agent hit the max iteration limit.";
          }
        }),
      );
    })();

    return () => unlistens.forEach((u) => u());
  });

  // Map tool_call_id -> tool result content for pairing cards with output.
  const resultsById = $derived.by(() => {
    const map = new Map<string, string>();
    for (const m of messages) {
      if (m.role === "tool" && m.tool_call_id) {
        map.set(m.tool_call_id, m.content ?? "");
      }
    }
    return map;
  });

  const visibleMessages = $derived(
    messages.filter((m) => m.role === "user" || m.role === "assistant"),
  );

  function apply(result: CoderRunResult) {
    messages = result.messages;
    pending = result.pending;
    if (result.exhausted) {
      error = "Agent hit the max iteration limit.";
    }
  }

  async function ensureThread(): Promise<CoderThread> {
    if (thread) return thread;
    const t = await coderService.createThread(workspaceRoot.trim());
    thread = t;
    messages = t.messages;
    await refreshThreads();
    return t;
  }

  async function send() {
    const text = input.trim();
    if (!text || busy) return;
    if (!workspaceRoot.trim()) {
      error = "Set a workspace folder first.";
      return;
    }
    error = null;
    busy = true;
    running = true;
    streamingText = "";
    try {
      const t = await ensureThread();
      input = "";
      // Optimistic echo; live events stream the rest, final result reconciles.
      messages = [...messages, { role: "user", content: text }];
      apply(await coderService.send(t.id, text));
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
      running = false;
    }
  }

  async function decide(
    approve: boolean,
    remember: boolean,
    editedPattern?: string,
  ) {
    if (!thread || !pending) return;
    const callId = pending.call_id;
    busy = true;
    running = true;
    streamingText = "";
    error = null;
    pending = null;
    try {
      const res = await coderService.approve(
        thread.id,
        callId,
        approve,
        remember,
        editedPattern,
      );
      apply(res);
      if (remember) rules = await coderService.listRules();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
      running = false;
    }
  }

  async function refreshThreads() {
    threads = await coderService.listThreads();
  }

  async function selectThread(id: string) {
    if (busy || running) return;
    const t = await coderService.getThread(id);
    if (!t) return;
    thread = t;
    messages = t.messages;
    workspaceRoot = t.workspace_root;
    pending = null;
    error = null;
  }

  function newSession() {
    if (busy || running) return;
    thread = null;
    messages = [];
    pending = null;
    error = null;
    workspaceRoot = "";
  }

  async function removeThread(id: string, e: MouseEvent) {
    e.stopPropagation();
    await coderService.deleteThread(id);
    if (thread?.id === id) newSession();
    await refreshThreads();
  }

  async function changeMode(next: PermissionMode) {
    mode = next;
    await coderService.setMode(next);
  }

  async function removeRule(r: PermissionRule) {
    await coderService.removeRule(r.tool, r.pattern);
    rules = await coderService.listRules();
  }

  function keydown(e: KeyboardEvent) {
    if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      send();
    }
  }

  function callsOf(m: ChatMessage): ToolCall[] {
    return m.tool_calls ?? [];
  }
</script>

<div class="flex h-full">
  <!-- Thread sidebar -->
  <aside class="flex w-56 shrink-0 flex-col border-r border-border">
    <div class="p-2">
      <Button
        size="sm"
        variant="secondary"
        class="w-full justify-start gap-2"
        onclick={newSession}
        disabled={busy || running}
      >
        <Plus class="h-4 w-4" /> New session
      </Button>
    </div>
    <div class="flex-1 overflow-auto px-2 pb-2">
      {#if threads.length === 0}
        <div class="px-1 py-2 text-xs text-muted-foreground">No sessions yet.</div>
      {/if}
      {#each threads as t}
        <button
          type="button"
          onclick={() => selectThread(t.id)}
          class="group mb-1 flex w-full items-center gap-2 rounded px-2 py-1.5 text-left text-xs {thread?.id ===
          t.id
            ? 'bg-primary/10 text-foreground'
            : 'text-muted-foreground hover:bg-muted'}"
        >
          <MessageSquare class="h-3.5 w-3.5 shrink-0" />
          <span class="flex-1 truncate">{t.title}</span>
          <span
            role="button"
            tabindex="0"
            class="opacity-0 group-hover:opacity-100 hover:text-destructive"
            onclick={(e) => removeThread(t.id, e)}
            onkeydown={(e) => e.key === "Enter" && removeThread(t.id, e as unknown as MouseEvent)}
          >
            <Trash2 class="h-3.5 w-3.5" />
          </span>
        </button>
      {/each}
    </div>
  </aside>

  <!-- Main column -->
  <div class="flex h-full flex-1 flex-col">
  <!-- Header: workspace + mode -->
  <div class="flex flex-wrap items-center gap-2 border-b border-border p-3">
    <Bot class="h-5 w-5 text-primary" />
    <span class="font-semibold">Coder</span>

    <input
      bind:value={workspaceRoot}
      disabled={!!thread}
      placeholder="workspace folder (absolute path)"
      class="flex-1 rounded border border-border bg-background px-2 py-1 font-mono text-xs"
    />

    <div class="flex items-center gap-1">
      {#each MODES as m}
        <button
          type="button"
          title={m.hint}
          onclick={() => changeMode(m.value)}
          class="rounded px-2 py-1 text-xs {mode === m.value
            ? 'bg-primary text-primary-foreground'
            : 'bg-muted text-muted-foreground'}"
        >
          {m.label}
        </button>
      {/each}
    </div>

    <Button
      size="icon"
      variant="ghost"
      title="Permission rules"
      onclick={() => (showSettings = !showSettings)}
    >
      <Settings2 class="h-4 w-4" />
    </Button>
  </div>

  {#if showSettings}
    <div class="border-b border-border bg-muted/30 p-3 text-sm">
      <div class="mb-2 font-medium">Allow / deny rules</div>
      {#if rules.length === 0}
        <div class="text-xs text-muted-foreground">
          No saved rules. "Accept &amp; remember" on an approval adds one.
        </div>
      {:else}
        <ul class="space-y-1">
          {#each rules as r}
            <li class="flex items-center gap-2 text-xs">
              <Badge variant={r.allow ? "secondary" : "destructive"}>
                {r.allow ? "allow" : "deny"}
              </Badge>
              <span class="font-mono">{r.tool}</span>
              <span class="font-mono text-muted-foreground">{r.pattern || "*"}</span>
              <button
                type="button"
                class="ml-auto text-muted-foreground hover:text-destructive"
                onclick={() => removeRule(r)}
              >
                <Trash2 class="h-3.5 w-3.5" />
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}

  <!-- Transcript -->
  <div class="flex-1 space-y-4 overflow-auto p-4">
    {#if visibleMessages.length === 0}
      <div class="mt-10 text-center text-sm text-muted-foreground">
        Describe a task. The agent explores, edits, and runs commands in your
        workspace — gated by the permission mode above.
      </div>
    {/if}

    {#each visibleMessages as m}
      <div class="flex gap-3">
        <div class="mt-0.5 shrink-0">
          {#if m.role === "user"}
            <User class="h-5 w-5 text-muted-foreground" />
          {:else}
            <Bot class="h-5 w-5 text-primary" />
          {/if}
        </div>
        <div class="min-w-0 flex-1 space-y-2">
          {#if m.content}
            <div class="whitespace-pre-wrap text-sm">{m.content}</div>
          {/if}
          {#each callsOf(m) as call}
            <ToolCallCard {call} result={resultsById.get(call.id) ?? null} />
          {/each}
        </div>
      </div>
    {/each}

    {#if streamingText}
      <div class="flex gap-3">
        <div class="mt-0.5 shrink-0"><Bot class="h-5 w-5 text-primary" /></div>
        <div class="min-w-0 flex-1 whitespace-pre-wrap text-sm">{streamingText}</div>
      </div>
    {/if}

    {#if running && !pending && !streamingText}
      <div class="flex items-center gap-2 text-sm text-muted-foreground">
        <Bot class="h-4 w-4 animate-pulse text-primary" />
        <span>working…</span>
      </div>
    {/if}

    {#if pending}
      <ApprovalBanner {pending} {busy} onDecision={decide} />
    {/if}

    {#if error}
      <div class="rounded border border-destructive/50 bg-destructive/10 p-2 text-xs text-destructive">
        {error}
      </div>
    {/if}
  </div>

  <!-- Composer -->
  <div class="border-t border-border p-3">
    <div class="flex items-end gap-2">
      <Textarea
        bind:value={input}
        onkeydown={keydown}
        rows={2}
        placeholder="Ask the coder to do something…  (Ctrl/Cmd+Enter to send)"
        class="flex-1 resize-none text-sm"
        disabled={busy || !!pending}
      />
      <Button onclick={send} disabled={busy || !!pending || !input.trim()}>
        <Send class="h-4 w-4" />
      </Button>
    </div>
  </div>
  </div>
</div>
