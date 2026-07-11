<script lang="ts">
  import { CoderPanel } from "$lib/domains/coder";
  import { afterNavigate } from "$app/navigation";
  import { coderSession } from "$lib/domains/coder/state/coderSession.svelte.js";
  import { isAiCoderRoute } from "$lib/config/ai-tabs";

  afterNavigate(async ({ to }) => {
    if (!to?.url.pathname || !isAiCoderRoute(to.url.pathname)) return;
    await coderSession.ensureInit();
    await coderSession.refreshThreads();
    await coderSession.syncRunningThreads();
  });
</script>

<div class="h-full min-h-0 overflow-hidden">
  <CoderPanel />
</div>
