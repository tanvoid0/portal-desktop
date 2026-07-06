<script lang="ts">
  import { CheckCircle, XCircle } from "@lucide/svelte";
  import { cn } from "$lib/utils.js";

  interface Props {
    installed: boolean;
    class?: string;
  }

  const { installed, class: className = "" }: Props = $props();
</script>

{#if installed}
  <CheckCircle
    class={cn("h-4 w-4 flex-shrink-0 text-green-500", className)}
  />
{:else}
  <XCircle class={cn("h-4 w-4 flex-shrink-0 text-red-500", className)} />
{/if}

