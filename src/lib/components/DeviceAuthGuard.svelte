<!--
	Device Authentication Guard
	Blocks access to the application from anywhere but the Tauri webview or localhost.

	Used to also let a remote browser authenticate with a passcode (verify_access_token +
	AuthScreen's device-passcode flow). That backend half is gone: REMOTE_ARCHITECTURE.md
	phase 3 decided against ever routing the webview over HTTP, and the daemon refuses any
	request carrying a browser `Origin` header, so there is no transport left for a remote
	browser to call anyway. Nothing replaces verify_access_token — a remote browser has
	nothing to authenticate into any more, so it just gets turned away.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { isTauriEnvironment } from '$lib/utils/tauri';
  import { InvokeClient } from '$lib/utils/invokeClient';
  import type { Snippet } from 'svelte';

  let { children }: { children: Snippet<[]> } = $props();

  let allowed = $state(false);
  let ready = $state(false);

  onMount(() => {
    allowed = isTauriEnvironment() || InvokeClient.isLocalhost();
    ready = true;
  });
</script>

{#if !ready}
  <div class="flex min-h-screen items-center justify-center">
    <div class="h-8 w-8 animate-spin rounded-full border-b-2 border-primary"></div>
  </div>
{:else if allowed}
  {@render children()}
{:else}
  <div class="flex min-h-screen items-center justify-center p-4">
    <p class="max-w-sm text-center text-sm text-muted-foreground">
      Portal Desktop can only be used from the desktop app.
    </p>
  </div>
{/if}
