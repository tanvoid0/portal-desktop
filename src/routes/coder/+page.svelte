<script lang="ts">
  import { CoderPanel } from "$lib/domains/coder";
  import { afterNavigate } from "$app/navigation";
  import { coderSession } from "$lib/domains/coder/state/coderSession.svelte.js";

  afterNavigate(async ({ to }) => {
    if (!to?.url.pathname.startsWith("/coder")) return;
    await coderSession.ensureInit();
    await coderSession.refreshThreads();
    await coderSession.syncRunningThreads();
  });
</script>

<div class="h-full min-h-0 overflow-hidden">
  <CoderPanel />
</div>
