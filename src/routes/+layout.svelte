<!--
	Root layout — QueryClientProvider + app shell
-->

<script lang="ts">
  // Must run before any invoke()/listen() call can fire — installs the browser
  // shim's window.__TAURI_INTERNALS__ when this is the thin-client build.
  import '$lib/domains/mobile/browserShim';
  import '../app.css';
  // Imported here rather than via app.css so Vite treats it as its own module —
  // the `devicon-woff-only` plugin in vite.config.ts needs to see it to strip
  // the eot/ttf/svg @font-face entries. Tailwind inlines app.css @imports first.
  import 'devicon/devicon.min.css';
  import '$lib/icons/offline.generated';
  import { onMount, type Snippet } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import QueryProvider from '$lib/components/providers/query-provider.svelte';
  import AppShell from '$lib/components/layout/app-shell.svelte';
  import ToastContainer from '$lib/components/ui/toast-container.svelte';
  import { toast } from '$lib/utils/toast';
  import { isMobileShell } from '$lib/domains/mobile/shell';
  import { mobileConnection } from '$lib/domains/mobile/connection.svelte';

  let { children }: { children: Snippet<[]> } = $props();

  // Portal Mobile: every route except /connect needs a live desktop connection.
  let wasConnected = false;
  onMount(() => {
    if (!isMobileShell) return;
    document.documentElement.classList.add('portal-mobile');
    void mobileConnection.init();
  });
  $effect(() => {
    if (!isMobileShell || !mobileConnection.loaded) return;
    const onConnect = page.url.pathname === '/connect';
    if (!mobileConnection.connected) {
      if (wasConnected) toast.warning('Disconnected from your PC');
      wasConnected = false;
      if (!onConnect) void goto('/connect', { replaceState: true });
    } else {
      wasConnected = true;
      if (onConnect) void goto('/', { replaceState: true });
    }
  });
</script>

<QueryProvider>
  {#if isMobileShell && !mobileConnection.connected}
    {#if mobileConnection.loaded && page.url.pathname === '/connect'}
      <div
        class="h-screen overflow-y-auto bg-background pt-[env(safe-area-inset-top)] pb-[env(safe-area-inset-bottom)]"
      >
        {@render children()}
      </div>
      <ToastContainer />
    {/if}
  {:else}
    <AppShell {children} />
  {/if}
</QueryProvider>
