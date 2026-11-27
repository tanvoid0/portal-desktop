<!-- Global floating stack of pending approvals from agent runs that aren't
     the session currently focused in the Agents workspace (#546) — so a
     blocked run stays visible from any page, not just while Agents is open.
     Mounted once from `app-shell.svelte`. -->
<script lang="ts">
  import { page } from '$app/state';
  import { createQuery } from '@tanstack/svelte-query';
  import ApprovalBanner from '$lib/domains/coder/components/ApprovalBanner.svelte';
  import type { PendingApproval } from '$lib/domains/coder/generated';
  import { agentCapabilities } from '$lib/domains/coder/config/capabilities.js';
  import { isAgentTabRoute } from '$lib/config/agents-tabs';
  import { threadStore } from '$lib/domains/threads/state/threadStore.svelte';
  import { agentRuns } from '../state/agentRuns.svelte.js';
  import { agentSessionsQuery } from '../queries/agentsQueries.js';

  const sessionsQuery = createQuery(agentSessionsQuery);
  const sessions = $derived(sessionsQuery.data ?? []);
  const focusedId = $derived(
    isAgentTabRoute(page.url.pathname) ? page.url.searchParams.get('session') : null
  );

  const stacks = $derived.by(() => {
    const out: {
      id: string;
      title: string;
      backendId: string;
      pending: PendingApproval[];
    }[] = [];
    for (const id of agentRuns.runningIds) {
      if (id === focusedId) continue;
      const pending = threadStore.view(id)?.approvals ?? [];
      if (pending.length === 0) continue;
      const summary = sessions.find((s) => s.id === id);
      out.push({
        id,
        title: summary?.title || summary?.backend_id || id,
        backendId: summary?.backend_id ?? '',
        pending,
      });
    }
    return out;
  });
</script>

{#if stacks.length > 0}
  <div
    class="fixed right-4 bottom-4 z-50 flex max-h-[60vh] w-[min(24rem,calc(100vw-2rem))] flex-col gap-2 overflow-y-auto"
  >
    {#each stacks as stack (stack.id)}
      <div class="overflow-hidden rounded-lg border bg-background shadow-lg">
        <div class="truncate border-b px-3 py-1.5 text-xs font-medium text-muted-foreground">
          {stack.title}
        </div>
        {#each stack.pending as approval (approval.call_id)}
          <ApprovalBanner
            pending={approval}
            showRemember={agentCapabilities(stack.backendId).rememberRule}
            onDecision={(approve) => agentRuns.decide(stack.id, approval.call_id, approve)}
          />
        {/each}
      </div>
    {/each}
  </div>
{/if}
