<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { ArrowRight, Globe } from "@lucide/svelte";
  import { coderWorkspaceStore } from "../state/coderWorkspaceStore.svelte.js";

  let url = $state(coderWorkspaceStore.browserUrl);

  function navigate() {
    const trimmed = url.trim();
    if (!trimmed) return;
    const withProtocol = /^https?:\/\//i.test(trimmed)
      ? trimmed
      : `https://${trimmed}`;
    coderWorkspaceStore.browserUrl = withProtocol;
    url = withProtocol;
  }
</script>

<div class="flex h-full min-h-0 flex-col">
  <div class="flex items-center gap-2 border-b border-border px-3 py-2">
    <Globe class="h-4 w-4 shrink-0 text-muted-foreground" />
    <Input
      bind:value={url}
      class="h-8 font-mono text-xs"
      placeholder="https://localhost:1420"
      onkeydown={(e) => e.key === "Enter" && navigate()}
    />
    <Button size="icon" variant="ghost" class="h-8 w-8 shrink-0" onclick={navigate}>
      <ArrowRight class="h-4 w-4" />
    </Button>
  </div>
  <div class="relative min-h-0 flex-1 bg-background">
    <iframe
      title="Workspace browser"
      src={coderWorkspaceStore.browserUrl}
      class="absolute inset-0 h-full w-full border-0"
      sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
    ></iframe>
  </div>
</div>
