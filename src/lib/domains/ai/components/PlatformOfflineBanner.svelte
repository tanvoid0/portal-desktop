<!-- Banner shown when agent-platform is unreachable or rejected the token, with retry/settings action. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { AI_PROVIDER_SETTINGS_PATH } from '$lib/config/ai-nav';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import ServerOff from '~icons/fa6-solid/server';

  interface Props {
    unauthorized: boolean;
    checking: boolean;
    error: string | null;
    onRetry: () => void;
  }

  let { unauthorized, checking, error, onRetry }: Props = $props();
</script>

<div
  class="divider-edge-b divider-edge-full flex flex-wrap items-center gap-3 bg-destructive/10 px-4 py-2.5 text-destructive"
  role="status"
>
  <ServerOff class="h-4 w-4 shrink-0" />
  <div class="min-w-0 flex-1">
    <p class="text-xs font-medium">
      {unauthorized ? 'Agent Platform rejected the API token' : 'Agent Platform is unreachable'}
    </p>
    <p class="truncate text-[11px] opacity-80" title={error ?? ''}>
      {unauthorized
        ? 'Update the token in Settings > AI, then reopen chat.'
        : (error ?? 'Check that the platform is running.')}
    </p>
  </div>
  {#if unauthorized}
    <Button
      size="sm"
      variant="outline"
      class="h-7 shrink-0 text-xs"
      href={AI_PROVIDER_SETTINGS_PATH}
    >
      Open Settings > AI
    </Button>
  {:else}
    <Button
      size="sm"
      variant="outline"
      class="h-7 shrink-0 gap-1.5 text-xs"
      disabled={checking}
      onclick={onRetry}
    >
      <RefreshCw class="h-3 w-3 {checking ? 'animate-spin' : ''}" />
      {checking ? 'Checking…' : 'Retry'}
    </Button>
  {/if}
</div>
