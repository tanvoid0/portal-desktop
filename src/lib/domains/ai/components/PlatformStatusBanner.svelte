<!--
  Honest agent-platform state: not_installed -> Download, unreachable -> Launch,
  unauthenticated -> Settings/pair, healthy (or unknown) -> renders nothing.
  `blocks` names what is disabled so the reason is visible next to the dead controls.
-->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { AI_PROVIDER_SETTINGS_PATH } from '$lib/config/ai-nav';
  import { openExternalUrl } from '$lib/utils/tauri';
  import { aiPlatform, AGENT_PLATFORM_RELEASES_URL } from '../state/aiPlatform.svelte.js';
  import ServerOff from '~icons/fa6-solid/server';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Loader2 from '~icons/fa6-solid/spinner';

  interface Props {
    /** What is disabled while this banner shows, e.g. "Sending". */
    blocks?: string;
  }
  let { blocks = 'Agent Platform features' }: Props = $props();

  const notInstalled = $derived(aiPlatform.status === 'not_installed');
  const unreachable = $derived(aiPlatform.status === 'unreachable');
</script>

{#if aiPlatform.blocked}
  <div
    class="divider-edge-b divider-edge-full flex flex-wrap items-center gap-3 bg-destructive/10 px-4 py-2.5 text-destructive"
    role="status"
    data-testid="platform-banner"
  >
    <ServerOff class="h-4 w-4 shrink-0" />
    <div class="min-w-0 flex-1">
      <p class="text-xs font-medium">
        {#if notInstalled}
          Agent Platform is not installed
        {:else if unreachable}
          Agent Platform is not running
        {:else}
          Agent Platform rejected this app’s token
        {/if}
      </p>
      <p class="truncate text-[11px] opacity-80" title={aiPlatform.detail ?? ''}>
        {blocks} unavailable.
        {#if notInstalled}
          Download and install it to continue.
        {:else if unreachable}
          Nothing is answering at {aiPlatform.baseUrl}.
        {:else}
          Pair again in Settings > AI.
        {/if}
      </p>
    </div>
    {#if notInstalled}
      <Button
        size="sm"
        variant="outline"
        class="h-7 shrink-0 text-xs"
        onclick={() => openExternalUrl(AGENT_PLATFORM_RELEASES_URL)}
      >
        Download latest release
      </Button>
    {:else if unreachable}
      <Button
        size="sm"
        variant="outline"
        class="h-7 shrink-0 gap-1.5 text-xs"
        disabled={aiPlatform.starting}
        onclick={() => aiPlatform.start()}
      >
        {#if aiPlatform.starting}<Loader2 class="h-3 w-3 animate-spin" />{/if}
        {aiPlatform.starting ? 'Launching…' : 'Launch Agent Platform'}
      </Button>
    {:else}
      <Button
        size="sm"
        variant="outline"
        class="h-7 shrink-0 text-xs"
        href={AI_PROVIDER_SETTINGS_PATH}
      >
        Open Settings > AI / Pair
      </Button>
    {/if}
    <Button
      size="sm"
      variant="outline"
      class="h-7 shrink-0 gap-1.5 text-xs"
      disabled={aiPlatform.checking || aiPlatform.starting}
      onclick={() => aiPlatform.refresh()}
    >
      <RefreshCw class="h-3 w-3 {aiPlatform.checking ? 'animate-spin' : ''}" />
      {aiPlatform.checking ? 'Checking…' : 'Retry'}
    </Button>
  </div>
{/if}
