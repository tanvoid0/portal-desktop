<!-- Agent platform reachability/auth badges with launch/download action. -->
<script lang="ts">
  import { Badge } from '$lib/components/ui/badge';
  import { Button } from '$lib/components/ui/button';
  import { openExternalUrl } from '$lib/utils/tauri';
  import { aiPlatform, AGENT_PLATFORM_RELEASES_URL } from '../../state/aiPlatform.svelte.js';
  import Loader2 from '~icons/fa6-solid/spinner';
</script>

<div
  class="flex flex-wrap items-center gap-2 text-xs text-muted-foreground"
  data-testid="platform-health"
>
  {#if aiPlatform.checking && !aiPlatform.known}
    Checking {aiPlatform.baseUrl || 'platform'}…
  {:else if aiPlatform.known}
    <span>{aiPlatform.baseUrl}</span>
    <Badge variant={aiPlatform.reachable ? 'secondary' : 'destructive'}>
      {aiPlatform.reachable ? 'reachable' : aiPlatform.installed ? 'not running' : 'not installed'}
    </Badge>
    <Badge variant="secondary">
      {aiPlatform.authRequired ? 'auth required' : 'no auth'}
    </Badge>
    {#if aiPlatform.reachable && aiPlatform.authRequired}
      <Badge variant={aiPlatform.authenticated ? 'secondary' : 'destructive'}>
        {aiPlatform.authenticated ? 'authenticated' : 'not authenticated'}
      </Badge>
    {/if}
    {#if aiPlatform.detail}<span class="truncate">{aiPlatform.detail}</span>{/if}
  {:else}
    Platform health unavailable.
  {/if}
  {#if aiPlatform.known && !aiPlatform.reachable}
    {#if aiPlatform.installed}
      <Button
        size="sm"
        variant="outline"
        class="h-6 text-xs"
        disabled={aiPlatform.starting}
        onclick={() => aiPlatform.start()}
      >
        {#if aiPlatform.starting}<Loader2 class="mr-1 h-3 w-3 animate-spin" />{/if}
        {aiPlatform.starting ? 'Launching…' : 'Launch Agent Platform'}
      </Button>
    {:else}
      <Button
        size="sm"
        variant="outline"
        class="h-6 text-xs"
        onclick={() => openExternalUrl(AGENT_PLATFORM_RELEASES_URL)}
      >
        Download latest release
      </Button>
    {/if}
  {/if}
</div>
