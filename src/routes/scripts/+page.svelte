<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  onMount(() => {
    goto("/automation/scripts", { replaceState: true });
  });
</script>

<svelte:head>
  <title>Scripts - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto p-6 text-muted-foreground">
  Redirecting to automation…
</div>
