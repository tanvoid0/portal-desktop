<!--
	Root layout — QueryClientProvider + app shell
-->

<script lang="ts">
  import "../app.css";
  import type { Snippet } from "svelte";
  import QueryProvider from "$lib/components/providers/query-provider.svelte";
  import AppShell from "$lib/components/layout/app-shell.svelte";

  let { children }: { children: Snippet<[]> } = $props();
</script>

<QueryProvider>
  <AppShell {children} />
</QueryProvider>
