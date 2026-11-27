<script lang="ts">
  import { CoderPanel } from '$lib/domains/coder';
  import { afterNavigate } from '$app/navigation';
  import { coderSession } from '$lib/domains/coder/state/coderSession.svelte.js';
  import { AI_CODER_PATH } from '$lib/config/ai-tabs';

  afterNavigate(async ({ to }) => {
    if (to?.url.pathname !== AI_CODER_PATH) return;
    await coderSession.ensureInit();
    await coderSession.refreshThreads();
  });
</script>

<div class="h-full min-h-0 overflow-hidden">
  <CoderPanel />
</div>
