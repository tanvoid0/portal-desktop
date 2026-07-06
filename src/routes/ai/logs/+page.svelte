<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  onMount(() => {
    goto("/ai/history", { replaceState: true });
  });
</script>

<svelte:head>
  <title>AI History - Portal Desktop</title>
</svelte:head>

<div class="flex items-center justify-center p-12 text-muted-foreground">
  Redirecting to conversation history…
</div>
