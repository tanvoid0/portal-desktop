<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  onMount(() => {
    goto("/automation/run", { replaceState: true });
  });
</script>

<svelte:head>
  <title>Automation - Portal Desktop</title>
</svelte:head>

<div class="py-12 text-center text-muted-foreground">Loading automation…</div>
