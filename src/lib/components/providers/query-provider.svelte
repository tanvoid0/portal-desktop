<script lang="ts">
  import { QueryClientProvider } from "@tanstack/svelte-query";
  import { queryClient } from "$lib/domains/shared/query/queryClient";
  import type { Snippet } from "svelte";

  let { children }: { children: Snippet<[]> } = $props();
</script>

<QueryClientProvider client={queryClient}>
  {@render children()}
</QueryClientProvider>
