<script lang="ts">
  import { onMount } from 'svelte';
  import { page } from '$app/stores';
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import { ScrollArea } from '$lib/components/ui/scroll-area/index.js';
  import {
    Bot,
    Settings2,
    Trash2,
    GitCompare,
    Sparkles,
    RotateCcw,
    FolderOpen,
    Square,
    Copy,
    Check,
  } from '@lucide/svelte';
  import { toast } from '$lib/utils/toast';
  import { goto, replaceState } from '$app/navigation';
  import { AI_PROVIDER_SETTINGS_PATH } from '$lib/config/ai-nav';
  import { coderSession } from '../state/coderSession.svelte.js';
  import type { ChatMessage, PermissionMode, PermissionRule, ToolCall } from '../types.js';
  import ToolCallCard from './ToolCallCard.svelte';
  import ApprovalBanner from './ApprovalBanner.svelte';
  import ChangesPanel from './ChangesPanel.svelte';
  import CoderProjectSelector from './CoderProjectSelector.svelte';
  import CoderSessionList from './CoderSessionList.svelte';
  import ProviderModelSelector from '$lib/domains/ai/components/ProviderModelSelector.svelte';
  import CoderComposer from './CoderComposer.svelte';
  import ChatMessageComponent from '$lib/domains/ai/components/chat/ChatMessage.svelte';
  import type { ChatMessage as AIChatMessage } from '$lib/domains/ai/types/index.js';

  let showSettings = $state(false);
  let showChanges = $state(false);
  let scrollViewport = $state<HTMLElement | null>(null);
  let input = $state('');
  let copied = $state(false);

  const MODES: { value: PermissionMode; label: string; hint: string }[] = [
    { value: 'review', label: 'Review', hint: 'prompt on each mutating action' },
    { value: 'auto-accept-all', label: 'Auto-accept', hint: 'run everything' },
    { value: 'plan', label: 'Plan', hint: 'read-only, no writes/commands' },
  ];

  const rt = $derived.by(() => {
    coderSession.runtimeRevision;
    return coderSession.activeRuntime;
  });
  const messages = $derived(rt.messages);
  const pending = $derived(rt.pending);
  const running = $derived(rt.running);
  const streamingText = $derived(rt.streamingText);
  const error = $derived(rt.error);
  const canRetry = $derived(rt.canRetry);
  const thread = $derived(coderSession.thread);
  const pendingChangeCount = $derived(
    coderSession.changes.filter((c) => c.status === 'pending').length,
  );
  const composerDisabled = $derived(!!pending);
  const visibleMessages = $derived(
    messages.filter((m) => m.role === 'user' || m.role === 'assistant'),
  );
  const isEmpty = $derived(
    visibleMessages.length === 0 && !streamingText && !running,
  );
  const workspaceName = $derived(
    coderSession.workspaceRoot.split(/[/\\]/).filter(Boolean).pop() ??
      coderSession.workspaceRoot,
  );
  const contextUsage = $derived(rt.contextUsage);
  const llmUsage = $derived(rt.llmUsage);
  const queuedMessages = $derived(rt.messageQueue);
  const runningCount = $derived(coderSession.runningThreadIds.size);
  const changesRevision = $derived(
    coderSession.changes.map((c) => `${c.id}:${c.status}`).join('|'),
  );

  onMount(async () => {
    await coderSession.ensureInit();

    const urlId = $page.url.searchParams.get('id');
    if (urlId) {
      const exists = coderSession.threads.find((t) => t.id === urlId);
      if (exists) await selectThread(exists.id);
      else {
        try {
          await coderSession.selectThread(urlId);
          syncUrl(urlId);
        } catch {
          /* ignore */
        }
      }
    }
  });

  function syncUrl(id: string | null) {
    const url = new URL($page.url);
    if (id) url.searchParams.set('id', id);
    else url.searchParams.delete('id');
    replaceState(url, {});
  }

  async function selectThread(id: string) {
    await coderSession.selectThread(id);
    input = coderSession.activeRuntime.draftInput;
    syncUrl(id);
  }

  function newSession() {
    coderSession.newSession();
    input = '';
    syncUrl(null);
  }

  async function send() {
    const text = input.trim();
    if (!text || composerDisabled) return;
    input = '';
    await coderSession.send(text);
    if (coderSession.activeThreadId) syncUrl(coderSession.activeThreadId);
  }

  function removeQueued(index: number) {
    const id = coderSession.activeThreadId;
    if (id) coderSession.removeFromQueue(id, index);
  }

  const resultsById = $derived.by(() => {
    const map = new Map<string, string>();
    for (const m of messages) {
      if (m.role === 'tool' && m.tool_call_id) {
        map.set(m.tool_call_id, m.content ?? '');
      }
    }
    return map;
  });

  $effect(() => {
    if (scrollViewport && (visibleMessages.length > 0 || streamingText || pending)) {
      requestAnimationFrame(() => {
        if (scrollViewport) scrollViewport.scrollTop = scrollViewport.scrollHeight;
      });
    }
  });

  $effect(() => {
    const id = coderSession.activeThreadId;
    if (id) input = coderSession.runtimeFor(id).draftInput;
  });

  function toDisplayMessage(m: ChatMessage): AIChatMessage {
    return { role: m.role as 'user' | 'assistant', content: m.content ?? '' };
  }

  function callsOf(m: ChatMessage): ToolCall[] {
    return m.tool_calls ?? [];
  }

  function toolNameFor(callId: string): string {
    for (const m of messages) {
      for (const tc of m.tool_calls ?? []) {
        if (tc.id === callId) return tc.function.name;
      }
    }
    return callId;
  }

  function formatConversationForCopy(): string {
    const lines: string[] = ['=== Coder Session ==='];
    if (thread?.title) lines.push(`Title: ${thread.title}`);
    if (coderSession.workspaceRoot) lines.push(`Workspace: ${coderSession.workspaceRoot}`);
    if (coderSession.selectedModel) lines.push(`Model: ${coderSession.selectedModel}`);
    lines.push(`Mode: ${coderSession.mode}`);
    if (thread?.id) lines.push(`Thread ID: ${thread.id}`);
    lines.push('');

    for (const m of messages) {
      if (m.role === 'system') continue;

      const roleLabel =
        m.role === 'user'
          ? 'User'
          : m.role === 'assistant'
            ? 'Assistant'
            : m.role === 'tool'
              ? `Tool (${toolNameFor(m.tool_call_id ?? '')})`
              : m.role;

      lines.push(`--- ${roleLabel} ---`);
      if (m.content) lines.push(m.content);

      for (const tc of m.tool_calls ?? []) {
        lines.push('');
        lines.push(`[Tool Call: ${tc.function.name}]`);
        lines.push(tc.function.arguments);
      }
      lines.push('');
    }

    if (streamingText) {
      lines.push('--- Assistant (streaming) ---');
      lines.push(streamingText);
      lines.push('');
    }

    if (pending) {
      lines.push('--- Pending Approval ---');
      lines.push(`Tool: ${pending.tool}`);
      lines.push(`Summary: ${pending.summary}`);
      lines.push(JSON.stringify(pending.arguments, null, 2));
      lines.push('');
    }

    if (error) {
      lines.push('--- Error ---');
      lines.push(error);
    }

    return lines.join('\n').trim();
  }

  const canCopyConversation = $derived(
    messages.some((m) => m.role !== 'system') || !!streamingText || !!pending || !!error,
  );

  async function copyConversation() {
    const text = formatConversationForCopy();
    if (!text) return;

    try {
      await navigator.clipboard.writeText(text);
      copied = true;
      toast.success('Conversation copied to clipboard');
      setTimeout(() => {
        copied = false;
      }, 2000);
    } catch {
      toast.error('Failed to copy conversation');
    }
  }
</script>

<div class="flex h-full w-full overflow-hidden">
  <aside class="flex w-64 flex-shrink-0 flex-col overflow-hidden border-r bg-background">
    <div class="border-b p-2.5">
      <h2 class="flex items-center gap-1.5 text-sm font-semibold">
        <Bot class="h-4 w-4" />
        Sessions
        {#if runningCount > 0}
          <span
            class="rounded-full bg-primary/15 px-1.5 py-0.5 text-[10px] font-medium text-primary"
            title="{runningCount} session(s) running"
          >
            {runningCount} active
          </span>
        {/if}
      </h2>
    </div>
    <CoderSessionList
      threads={coderSession.threads}
      onThreadClick={(t) => selectThread(t.id)}
      onCreateNew={newSession}
      onDeleteThread={(t) => coderSession.removeThread(t)}
      selectedThreadId={coderSession.activeThreadId}
      runningThreadIds={coderSession.runningThreadIds}
      queuedCountFor={(id) => {
        coderSession.runtimeRevision;
        return coderSession.queuedCountFor(id);
      }}
    />
  </aside>

  <main class="flex min-w-0 flex-1 flex-col overflow-hidden">
    <div
      class="flex flex-wrap items-center justify-between gap-2 border-b bg-background px-4 py-2.5"
    >
      <div class="flex min-w-0 items-center gap-2">
        <h2 class="truncate text-sm font-semibold">
          {thread?.title || 'New session'}
        </h2>
        {#if running}
          <span class="flex items-center gap-1 text-xs text-primary">
            <span class="h-1.5 w-1.5 animate-pulse rounded-full bg-primary"></span>
            Running
          </span>
        {/if}
        {#if queuedMessages.length > 0}
          <span class="text-xs text-muted-foreground">
            {queuedMessages.length} queued
          </span>
        {/if}
        {#if coderSession.workspaceRoot}
          <span class="hidden items-center gap-1 text-xs text-muted-foreground sm:flex">
            <FolderOpen class="h-3 w-3" />
            <span class="max-w-[200px] truncate">{workspaceName}</span>
          </span>
        {/if}
      </div>

      <div class="flex flex-wrap items-center gap-1.5">
        <CoderProjectSelector
          bind:value={coderSession.workspaceRoot}
          disabled={!!thread}
        />
        <ProviderModelSelector
          bind:selectedProvider={coderSession.selectedProvider}
          bind:selectedModel={coderSession.selectedModel}
          onModelChange={(m) => coderSession.handleModelChange(m)}
          modelSelectClass="w-[180px]"
        />
        <div class="flex items-center rounded-md border border-border p-0.5">
          {#each MODES as m}
            <button
              type="button"
              title={m.hint}
              onclick={() => coderSession.changeMode(m.value)}
              class="rounded px-2 py-0.5 text-xs transition-colors {coderSession.mode ===
              m.value
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:bg-muted'}"
            >
              {m.label}
            </button>
          {/each}
        </div>
        {#if running}
          <Button
            size="sm"
            variant="destructive"
            class="h-8 gap-1"
            onclick={() => coderSession.stop()}
          >
            <Square class="h-3 w-3 fill-current" />
            Stop
          </Button>
        {/if}
        <Button
          size="icon"
          variant="ghost"
          class="h-8 w-8"
          title="Copy conversation for debugging"
          disabled={!canCopyConversation}
          onclick={copyConversation}
        >
          {#if copied}
            <Check class="h-3.5 w-3.5 text-green-500" />
          {:else}
            <Copy class="h-3.5 w-3.5" />
          {/if}
        </Button>
        <Button
          size="sm"
          variant={showChanges ? 'secondary' : 'ghost'}
          class="h-8 gap-1"
          title="Review file changes"
          onclick={() => {
            showChanges = !showChanges;
            if (showChanges) coderSession.refreshChanges(thread?.id);
          }}
        >
          <GitCompare class="h-3.5 w-3.5" />
          {#if pendingChangeCount > 0}
            <span class="rounded-full bg-amber-500 px-1.5 text-[10px] text-white">
              {pendingChangeCount}
            </span>
          {/if}
        </Button>
        <Button
          size="icon"
          variant="ghost"
          class="h-8 w-8"
          title="AI provider settings"
          onclick={() => goto(AI_PROVIDER_SETTINGS_PATH)}
        >
          <Sparkles class="h-3.5 w-3.5" />
        </Button>
        <Button
          size="icon"
          variant="ghost"
          class="h-8 w-8"
          title="Permission rules"
          onclick={() => (showSettings = !showSettings)}
        >
          <Settings2 class="h-3.5 w-3.5" />
        </Button>
      </div>
    </div>

    {#if showChanges}
      <div class="max-h-72 overflow-auto border-b border-border bg-muted/30 p-3">
        <ChangesPanel
          changes={coderSession.changes}
          onRefresh={() => coderSession.refreshChanges(thread?.id)}
        />
      </div>
    {/if}

    {#if showSettings}
      <div class="border-b border-border bg-muted/30 p-3 text-sm">
        <div class="mb-2 font-medium">Allow / deny rules</div>
        {#if coderSession.rules.length === 0}
          <div class="text-xs text-muted-foreground">
            No saved rules. "Accept &amp; remember" on an approval adds one.
          </div>
        {:else}
          <ul class="space-y-1">
            {#each coderSession.rules as r}
              <li class="flex items-center gap-2 text-xs">
                <Badge variant={r.allow ? 'secondary' : 'destructive'}>
                  {r.allow ? 'allow' : 'deny'}
                </Badge>
                <span class="font-mono">{r.tool}</span>
                <span class="font-mono text-muted-foreground">{r.pattern || '*'}</span>
                <button
                  type="button"
                  class="ml-auto text-muted-foreground hover:text-destructive"
                  onclick={() => coderSession.removeRule(r)}
                >
                  <Trash2 class="h-3.5 w-3.5" />
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      </div>
    {/if}

    {#if isEmpty}
      <div class="flex flex-1 flex-col items-center justify-center px-4 pb-8">
        <div class="mb-6 text-center">
          <div
            class="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-primary/10"
          >
            <Bot class="h-6 w-6 text-primary" />
          </div>
          <h3 class="text-lg font-semibold">What should we build?</h3>
          <p class="mt-1 max-w-md text-sm text-muted-foreground">
            Describe a task. Agent explores, edits, and runs commands in your workspace — gated by
            permission mode.
          </p>
        </div>
        <div class="w-full max-w-3xl">
          <CoderComposer
            bind:value={input}
            onSend={send}
            rows={3}
            placeholder="Ask the coder to do something…"
            disabled={composerDisabled}
            {running}
            {queuedMessages}
            onRemoveQueued={removeQueued}
            mode={coderSession.mode}
            onModeChange={(m) => coderSession.changeMode(m)}
            workspaceRoot={coderSession.workspaceRoot}
            onToggleChanges={() => {
              showChanges = !showChanges;
              if (showChanges) coderSession.refreshChanges(thread?.id);
            }}
            {showChanges}
            {changesRevision}
            {contextUsage}
            {llmUsage}
            class="px-0 pb-0"
          />
        </div>
        {#if error}
          <div
            class="mt-4 flex max-w-lg items-start gap-2 rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-xs text-destructive"
          >
            <span class="flex-1">{error}</span>
          </div>
        {/if}
      </div>
    {:else}
      <ScrollArea class="min-h-0 flex-1" bind:viewportRef={scrollViewport}>
        <div class="mx-auto max-w-3xl space-y-4 px-4 py-6">
          {#each visibleMessages as m}
            {#if callsOf(m).length > 0}
              <ChatMessageComponent message={toDisplayMessage(m)}>
                {#snippet children()}
                  {#each callsOf(m) as call}
                    <ToolCallCard
                      {call}
                      result={resultsById.get(call.id) ?? null}
                      workspaceRoot={thread?.workspace_root ?? coderSession.workspaceRoot}
                    />
                  {/each}
                {/snippet}
              </ChatMessageComponent>
            {:else}
              <ChatMessageComponent message={toDisplayMessage(m)} />
            {/if}
          {/each}
          {#if streamingText}
            <ChatMessageComponent message={{ role: 'assistant', content: streamingText }} />
          {/if}
          {#if running && !pending && !streamingText}
            <ChatMessageComponent message={{ role: 'assistant', content: '' }} showLoader={true} />
          {/if}
          {#if pending}
            <ApprovalBanner
              {pending}
              busy={running}
              onDecision={(a, r, p) => coderSession.decide(a, r, p)}
            />
          {/if}
          {#if error}
            <div
              class="flex items-start gap-2 rounded-lg border border-destructive/50 bg-destructive/10 p-3 text-xs text-destructive"
            >
              <span class="flex-1">{error}</span>
              {#if canRetry && thread}
                <Button
                  size="sm"
                  variant="outline"
                  class="h-7 shrink-0 gap-1 border-destructive/40 text-destructive hover:bg-destructive/10"
                  onclick={() => coderSession.retry()}
                  disabled={running}
                >
                  <RotateCcw class="h-3.5 w-3.5" />
                  Retry
                </Button>
              {/if}
            </div>
          {/if}
        </div>
      </ScrollArea>
      <div class="shrink-0 bg-gradient-to-t from-background via-background to-transparent pt-2">
        <CoderComposer
          bind:value={input}
          onSend={send}
          disabled={composerDisabled}
          {running}
          {queuedMessages}
          onRemoveQueued={removeQueued}
          mode={coderSession.mode}
          onModeChange={(m) => coderSession.changeMode(m)}
          workspaceRoot={coderSession.workspaceRoot}
          onToggleChanges={() => {
            showChanges = !showChanges;
            if (showChanges) coderSession.refreshChanges(thread?.id);
          }}
          {showChanges}
          {changesRevision}
          {contextUsage}
          {llmUsage}
          class="pt-0"
        />
      </div>
    {/if}
  </main>
</div>
