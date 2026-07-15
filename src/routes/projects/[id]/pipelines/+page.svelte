<!--
  Legacy pipeline management route — redirects to the unified Actions tab.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { page } from "$app/stores";
  import { goto } from "$app/navigation";
  import { PageLoading } from "$lib/components/shell";

  onMount(() => {
    const id = $page.params.id;
    if (id) {
      void goto(`/projects/${id}?tab=actions`, { replaceState: true });
    } else {
      void goto("/projects", { replaceState: true });
    }
  });
</script>

<PageLoading message="Opening Actions…" />
