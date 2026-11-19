<!--
  Daemon status snapshot: server/version, connection, uptime, access token,
  readiness probes and in-flight process counts. Fields come from the daemon's
  free-form /system/status; everything degrades to "—" when absent.
-->
<script lang="ts" module>
  function formatUptime(seconds?: number): string {
    if (seconds == null || !Number.isFinite(seconds)) return '—';
    const s = Math.max(0, Math.floor(seconds));
    const d = Math.floor(s / 86400);
    const h = Math.floor((s % 86400) / 3600);
    const m = Math.floor((s % 3600) / 60);
    const sec = s % 60;
    const parts: string[] = [];
    if (d) parts.push(`${d}d`);
    if (h || d) parts.push(`${h}h`);
    if (m || h || d) parts.push(`${m}m`);
    parts.push(`${sec}s`);
    return parts.join(' ');
  }

  function maskToken(token: string): string {
    if (token.length <= 10) return '•'.repeat(Math.max(4, token.length));
    return `${token.slice(0, 6)}${'•'.repeat(8)}${token.slice(-4)}`;
  }
</script>

<script lang="ts">
  import { Card } from '$lib/components/ui/card';
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { toast } from '$lib/utils/toast';
  import { aiPlatform } from '../../state/aiPlatform.svelte.js';
  import type { PlatformSystemStatus } from '../../services/platformSystemService.js';
  import Copy from '~icons/fa6-solid/copy';
  import Eye from '~icons/fa6-solid/eye';
  import EyeSlash from '~icons/fa6-solid/eye-slash';

  interface Props {
    status?: PlatformSystemStatus;
    /** The saved `agp_` token, shown masked with a copy control. */
    token?: string | null;
    pending?: boolean;
  }
  let { status, token = null, pending = false }: Props = $props();

  let revealed = $state(false);

  const port = $derived(status?.listening_on?.port);
  const readinessOk = $derived(status?.readiness?.ok);
  const proxyOk = $derived(status?.llm_proxy?.ok);
  const checks = $derived.by(() => [
    ...(status?.readiness?.checks ?? []),
    ...(status?.llm_proxy?.checks ?? []),
  ]);
  const byStatus = $derived.by(() => Object.entries(status?.processes?.by_status ?? {}));

  async function copyToken() {
    if (!token) return;
    try {
      await navigator.clipboard.writeText(token);
      toast.success('Token copied to clipboard');
    } catch (e) {
      toast.error('Could not copy token', e);
    }
  }
</script>

{#snippet dot(ok: boolean | undefined)}
  <span
    class="inline-block h-2 w-2 shrink-0 rounded-full {ok === undefined
      ? 'bg-amber-500'
      : ok
        ? 'bg-green-500'
        : 'bg-red-500'}"
  ></span>
{/snippet}

<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
  <!-- Server -->
  <Card class="p-4">
    <div class="text-xs font-medium tracking-wide text-muted-foreground uppercase">Server</div>
    <div class="mt-2 flex items-center gap-2">
      <span class="text-lg font-semibold">{status?.service ?? 'agent-platform'}</span>
      {#if status?.env}
        <Badge variant="outline" class="capitalize">{status.env}</Badge>
      {/if}
    </div>
    <dl class="mt-2 space-y-1 text-sm">
      <div class="flex justify-between gap-3">
        <dt class="text-muted-foreground">Version</dt>
        <dd class="font-mono">{status?.server ?? '—'}</dd>
      </div>
      <div class="flex justify-between gap-3">
        <dt class="text-muted-foreground">Platform</dt>
        <dd class="truncate font-mono" title={status?.platform ?? ''}>{status?.platform ?? '—'}</dd>
      </div>
    </dl>
  </Card>

  <!-- Connection -->
  <Card class="p-4">
    <div class="text-xs font-medium tracking-wide text-muted-foreground uppercase">Connection</div>
    <div class="mt-2 truncate font-mono text-sm" title={aiPlatform.baseUrl}>
      {aiPlatform.baseUrl || '—'}
    </div>
    <dl class="mt-2 space-y-1 text-sm">
      <div class="flex items-center justify-between gap-3">
        <dt class="text-muted-foreground">Reachable</dt>
        <dd class="flex items-center gap-2">
          {@render dot(aiPlatform.known ? aiPlatform.reachable : undefined)}
          {aiPlatform.known ? (aiPlatform.reachable ? 'Yes' : 'No') : 'Checking…'}
        </dd>
      </div>
      <div class="flex items-center justify-between gap-3">
        <dt class="text-muted-foreground">Authenticated</dt>
        <dd class="flex items-center gap-2">
          {@render dot(aiPlatform.known ? aiPlatform.authenticated : undefined)}
          {aiPlatform.authenticated
            ? 'Yes'
            : status?.auth_required === false
              ? 'Not required'
              : 'No'}
        </dd>
      </div>
      <div class="flex justify-between gap-3">
        <dt class="text-muted-foreground">Port</dt>
        <dd class="font-mono">{port ?? '—'}</dd>
      </div>
    </dl>
  </Card>

  <!-- Uptime -->
  <Card class="p-4">
    <div class="text-xs font-medium tracking-wide text-muted-foreground uppercase">Uptime</div>
    <div class="mt-2 text-lg font-semibold">{formatUptime(status?.uptime_seconds)}</div>
    <p class="mt-1 text-xs text-muted-foreground">Since the daemon last started.</p>
  </Card>

  <!-- Access token -->
  <Card class="p-4">
    <div class="text-xs font-medium tracking-wide text-muted-foreground uppercase">
      Access token
    </div>
    {#if token}
      <div class="mt-2 flex items-center gap-1.5">
        <code class="min-w-0 flex-1 truncate rounded bg-muted px-2 py-1 font-mono text-xs">
          {revealed ? token : maskToken(token)}
        </code>
        <Button
          variant="ghost"
          size="icon"
          class="h-7 w-7 shrink-0"
          aria-label={revealed ? 'Hide token' : 'Reveal token'}
          onclick={() => (revealed = !revealed)}
        >
          {#if revealed}<EyeSlash class="h-3.5 w-3.5" />{:else}<Eye class="h-3.5 w-3.5" />{/if}
        </Button>
        <Button
          variant="ghost"
          size="icon"
          class="h-7 w-7 shrink-0"
          aria-label="Copy token"
          onclick={copyToken}
        >
          <Copy class="h-3.5 w-3.5" />
        </Button>
      </div>
      <p class="mt-1 text-xs text-muted-foreground">
        The workspace token portal uses for this daemon.
      </p>
    {:else}
      <p class="mt-2 text-sm text-muted-foreground">
        No token saved. Pair in Settings → AI → Providers.
      </p>
    {/if}
  </Card>

  <!-- Readiness -->
  <Card class="p-4">
    <div class="text-xs font-medium tracking-wide text-muted-foreground uppercase">Readiness</div>
    <div class="mt-2 flex flex-wrap gap-4 text-sm">
      <div class="flex items-center gap-2">
        {@render dot(readinessOk)}
        <span>App</span>
      </div>
      <div class="flex items-center gap-2">
        {@render dot(proxyOk)}
        <span>LLM proxy</span>
      </div>
    </div>
    {#if checks.length}
      <ul class="mt-2 space-y-1">
        {#each checks as check (check.name)}
          <li class="flex items-start gap-2 text-xs" title={check.detail}>
            {@render dot(check.ok)}
            <span class="min-w-0 flex-1 truncate text-muted-foreground">{check.name}</span>
          </li>
        {/each}
      </ul>
    {:else if pending}
      <p class="mt-2 text-xs text-muted-foreground">Loading…</p>
    {/if}
  </Card>

  <!-- Processes -->
  <Card class="p-4">
    <div class="text-xs font-medium tracking-wide text-muted-foreground uppercase">Processes</div>
    <div class="mt-2 flex items-baseline gap-2">
      <span class="text-lg font-semibold">{status?.processes?.active ?? 0}</span>
      <span class="text-sm text-muted-foreground">in flight</span>
      <span class="ml-auto text-sm text-muted-foreground">
        {status?.processes?.total ?? 0} total
      </span>
    </div>
    {#if byStatus.length}
      <div class="mt-2 flex flex-wrap gap-1.5">
        {#each byStatus as [name, count] (name)}
          <Badge variant="secondary" class="font-normal">{name}: {count}</Badge>
        {/each}
      </div>
    {:else}
      <p class="mt-2 text-xs text-muted-foreground">No processes recorded.</p>
    {/if}
  </Card>
</div>
