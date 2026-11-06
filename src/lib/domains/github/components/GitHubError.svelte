<script lang="ts">
  import { goto } from '$app/navigation';
  import { PageEmpty, PageError } from '$lib/components/shell';
  import KeyRound from '~icons/fa6-solid/key';
  import { isTokenUnreadableError } from '../utils/tokenError';

  interface Props {
    title?: string;
    message: string;
    onRetry?: () => void;
  }

  let { title, message, onRetry }: Props = $props();
</script>

{#if isTokenUnreadableError(message)}
  <PageEmpty
    title="Stored GitHub token can't be read"
    description="Portal can't decrypt the saved GitHub token. Reconnect GitHub to continue."
    icon={KeyRound}
    actionLabel="Reconnect GitHub"
    onAction={() => goto('/settings/github')}
  />
{:else}
  <PageError {title} {message} {onRetry} />
{/if}
