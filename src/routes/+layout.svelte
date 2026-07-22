<!--
	Root layout — QueryClientProvider + app shell
-->

<script lang="ts">
  import "../app.css";
  // Imported here rather than via app.css so Vite treats it as its own module —
  // the `devicon-woff-only` plugin in vite.config.ts needs to see it to strip
  // the eot/ttf/svg @font-face entries. Tailwind inlines app.css @imports first.
  import "devicon/devicon.min.css";
  import type { Snippet } from "svelte";
  import QueryProvider from "$lib/components/providers/query-provider.svelte";
  import AppShell from "$lib/components/layout/app-shell.svelte";

  let { children }: { children: Snippet<[]> } = $props();
</script>

<QueryProvider>
  <AppShell {children} />
</QueryProvider>
