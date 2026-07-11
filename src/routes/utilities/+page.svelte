<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";

  onMount(() => {
    goto("/utilities/disk", { replaceState: true });
  });
</script>

<div class="text-sm text-muted-foreground">Opening utilities…</div>
