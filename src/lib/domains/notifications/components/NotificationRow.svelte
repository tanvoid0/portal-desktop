<!-- One inbox entry: what happened, where, and what can be done about it. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Badge } from '$lib/components/ui/badge';
  import ApprovalBanner from '$lib/domains/coder/components/ApprovalBanner.svelte';
  import { agentRuns } from '$lib/domains/agents/state/agentRuns.svelte';
  import type { AgentSessionSummary } from '$lib/domains/agents/generated/AgentSessionSummary';
  import type { Project } from '$lib/domains/projects/types';
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import { toast } from '$lib/utils/toast';
  import { formatRelativeTime } from '$lib/domains/shared/utils';
  import { isMobileShell } from '$lib/domains/mobile/shell';
  import type { FactKind, InboxItem } from '../state/notificationInbox.svelte';
  import Robot from '~icons/fa6-solid/robot';
  import DiagramProject from '~icons/fa6-solid/diagram-project';
  import Mobile from '~icons/fa6-solid/mobile-screen-button';
  import Bell from '~icons/fa6-solid/bell';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import Cube from '~icons/fa6-solid/cube';
  import Wrench from '~icons/fa6-solid/wrench';
  import Terminal from '~icons/fa6-solid/terminal';
  import FileLines from '~icons/fa6-solid/file-lines';
  import ListCheck from '~icons/fa6-solid/list-check';
  import Hashtag from '~icons/fa6-solid/hashtag';
  import Stopwatch from '~icons/fa6-solid/stopwatch';
  import Clock from '~icons/fa6-solid/clock';
  import ArrowUpRight from '~icons/fa6-solid/arrow-up-right-from-square';
  import Check from '~icons/fa6-solid/check';
  import Xmark from '~icons/fa6-solid/xmark';

  interface Props {
    item: InboxItem;
    session?: AgentSessionSummary;
    project?: Project;
    onOpen: (item: InboxItem) => void;
    onMarkRead: (id: string) => void;
    onRemove: (id: string) => void;
  }

  let { item, session, project, onOpen, onMarkRead, onRemove }: Props = $props();

  const SOURCES = {
    agent: { label: 'Agent', icon: Robot, open: 'Open session' },
    pipeline: { label: 'Pipeline', icon: DiagramProject, open: 'View run' },
    device: { label: 'Device', icon: Mobile, open: 'Review device' },
  };
  const TONES = {
    success: 'bg-green-500/10 text-green-600 dark:text-green-500',
    error: 'bg-destructive/10 text-destructive',
    warning: 'bg-amber-500/10 text-amber-600 dark:text-amber-500',
    info: 'bg-muted text-muted-foreground',
  };
  const FACTS: Record<FactKind, { label: string; icon: typeof Wrench }> = {
    tool: { label: 'Tool', icon: Wrench },
    command: { label: 'Command', icon: Terminal },
    file: { label: 'File', icon: FileLines },
    step: { label: 'Failed step', icon: ListCheck },
    exit: { label: 'Exit code', icon: Hashtag },
    duration: { label: 'Duration', icon: Stopwatch },
    device: { label: 'Device', icon: Mobile },
  };

  const source = $derived(item.source ? SOURCES[item.source] : undefined);
  const SourceIcon = $derived(source?.icon ?? Bell);
  const folder = $derived(session?.cwd ?? project?.path);

  const chips = $derived.by(() => {
    const out: { label: string; icon: typeof Wrench; value: string }[] = [];
    if (session) {
      const model = session.model ? ` · ${session.model}` : '';
      out.push({ label: 'Agent', icon: Robot, value: `${session.backend_id}${model}` });
    }
    if (project) out.push({ label: 'Project', icon: Cube, value: project.name });
    if (folder) out.push({ label: 'Folder', icon: FolderOpen, value: folder });
    for (const fact of item.facts ?? []) {
      const meta = FACTS[fact.kind];
      if (meta) out.push({ ...meta, value: fact.value });
    }
    return out;
  });

  // Only while the run is still blocked on it; answered or dead requests get no buttons.
  const approval = $derived(
    item.runId && item.requestId && agentRuns.runningIds.has(item.runId)
      ? agentRuns.viewFor(item.runId).pending.find((p) => p.call_id === item.requestId)
      : undefined
  );

  async function decide(approve: boolean) {
    try {
      await agentRuns.decide(item.runId!, approval!.call_id, approve);
      onMarkRead(item.id);
    } catch (err) {
      toast.error('Could not send your decision', err);
    }
  }

  function showFolder(path: string) {
    // `revealItemInDir`: `opener:default` allows it, and not `openPath`.
    revealItemInDir(path).catch((err) => toast.error('Could not show the folder', err));
  }
</script>

<li class="flex gap-3 p-3">
  <span
    class="flex h-8 w-8 shrink-0 items-center justify-center rounded-md {TONES[item.type] ??
      TONES.info}"
    title={source?.label}
  >
    <SourceIcon class="h-4 w-4" />
    {#if source}<span class="sr-only">{source.label}</span>{/if}
  </span>

  <div class="flex min-w-0 flex-1 flex-col gap-2">
    <div class="flex items-start gap-2">
      <div class="min-w-0 flex-1">
        <p class="flex items-center gap-2 text-sm">
          <span class="min-w-0 break-words {item.read ? '' : 'font-semibold'}">{item.title}</span>
          {#if !item.read}
            <span class="h-2 w-2 shrink-0 rounded-full bg-primary" aria-hidden="true"></span>
            <span class="sr-only">Unread</span>
          {/if}
        </p>
        <time
          class="flex items-center gap-1 text-xs text-muted-foreground"
          datetime={new Date(item.at).toISOString()}
          title={new Date(item.at).toLocaleString()}
        >
          <Clock class="h-3 w-3" aria-hidden="true" />
          {formatRelativeTime(new Date(item.at))}
        </time>
      </div>
      <Button
        variant="ghost"
        size="icon"
        class="h-7 w-7 shrink-0"
        aria-label="Remove notification: {item.title}"
        title="Remove"
        onclick={() => onRemove(item.id)}
      >
        <Xmark class="h-3.5 w-3.5" />
      </Button>
    </div>

    {#if session?.title}
      <p class="truncate text-sm font-medium" title={session.title}>{session.title}</p>
    {/if}
    {#if item.body}
      <p class="line-clamp-2 text-sm break-words text-muted-foreground">{item.body}</p>
    {/if}

    {#if chips.length}
      <ul class="flex flex-wrap gap-1.5" aria-label="Details">
        {#each chips as chip, n (n)}
          <li class="max-w-full min-w-0">
            <Badge
              variant="outline"
              class="max-w-full gap-1.5 font-normal"
              title="{chip.label}: {chip.value}"
            >
              <chip.icon class="shrink-0 text-muted-foreground" aria-hidden="true" />
              <span class="sr-only">{chip.label}:</span>
              <span class="truncate">{chip.value}</span>
            </Badge>
          </li>
        {/each}
      </ul>
    {/if}

    {#if approval}
      <div class="overflow-hidden rounded-md border">
        <!-- No "remember" here: `agentRuns.decide` has nowhere to put a rule. -->
        <ApprovalBanner pending={approval} showRemember={false} onDecision={decide} />
      </div>
    {/if}

    <div class="flex flex-wrap gap-2">
      {#if item.route}
        <Button size="sm" variant="secondary" class="gap-1.5" onclick={() => onOpen(item)}>
          <ArrowUpRight class="h-3 w-3" />
          {source?.open ?? 'Open'}
        </Button>
      {/if}
      {#if folder && !isMobileShell}
        <Button size="sm" variant="outline" class="gap-1.5" onclick={() => showFolder(folder)}>
          <FolderOpen class="h-3 w-3" />
          Show folder
        </Button>
      {/if}
      {#if !item.read}
        <Button size="sm" variant="ghost" class="gap-1.5" onclick={() => onMarkRead(item.id)}>
          <Check class="h-3 w-3" />
          Mark read
        </Button>
      {/if}
    </div>
  </div>
</li>
