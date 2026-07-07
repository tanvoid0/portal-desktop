<!--
	AI Providers - Redirects to settings
-->

<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";

  onMount(() => {
    goto("/settings/ai", { replaceState: true });
  });
</script>

<div class="flex items-center justify-center py-12">
  <p class="text-muted-foreground">Redirecting to AI provider settings...</p>
</div>
