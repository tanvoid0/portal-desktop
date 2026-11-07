<!--
	Pipeline Run Detail - status, duration, step timeline and log output for one execution.
	Route-agnostic: it reads no route params and owns no page chrome, so it works on a
	full page or inside a rail. Navigation leaves through onClose.
-->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import ArrowLeft from '~icons/fa6-solid/arrow-left';
  import ExecutionMonitor from './ExecutionMonitor.svelte';

  interface Props {
    executionId: string;
    /** Dismiss the detail — wired to the Close action and the optional back button. */
    onClose?: () => void;
    /** When set (with onClose), a back button with this label sits above the run. */
    backLabel?: string;
  }

  let { executionId, onClose, backLabel }: Props = $props();
</script>

<div class="flex h-full min-h-0 flex-col gap-4">
  {#if backLabel && onClose}
    <Button variant="ghost" size="sm" onclick={onClose} class="w-fit gap-2">
      <ArrowLeft class="h-4 w-4" />
      {backLabel}
    </Button>
  {/if}

  {#if executionId}
    <!-- ExecutionMonitor loads and subscribes in onMount, so a different run needs a
		 fresh instance rather than a changed prop -->
    {#key executionId}
      <ExecutionMonitor {executionId} {onClose} />
    {/key}
  {:else}
    <p class="text-muted-foreground">Invalid execution ID</p>
  {/if}
</div>
