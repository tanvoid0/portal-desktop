<script lang="ts">
  import ChatMarkdown from "./ChatMarkdown.svelte";
  import { Button } from "../button";

  interface Props {
    content: string;
    /** Start in rendered or raw source view. */
    defaultView?: "rendered" | "source";
    class?: string;
  }

  let {
    content,
    defaultView = "rendered",
    class: className = "",
  }: Props = $props();

  let view = $state<"rendered" | "source">(defaultView);
</script>

<div class="space-y-2 {className}">
  <div class="flex items-center gap-1">
    <Button
      type="button"
      variant="ghost"
      size="sm"
      class="h-auto rounded px-2 py-0.5 text-[10px] font-medium {view === 'rendered'
        ? 'bg-primary/15 text-primary'
        : 'text-muted-foreground hover:text-foreground'}"
      onclick={() => (view = "rendered")}
    >
      Rendered
    </Button>
    <Button
      type="button"
      variant="ghost"
      size="sm"
      class="h-auto rounded px-2 py-0.5 text-[10px] font-medium {view === 'source'
        ? 'bg-primary/15 text-primary'
        : 'text-muted-foreground hover:text-foreground'}"
      onclick={() => (view = "source")}
    >
      Source
    </Button>
  </div>

  {#if view === "rendered"}
    <div
      class="max-h-[28rem] overflow-auto rounded border border-border bg-background p-3"
    >
      <ChatMarkdown {content} />
    </div>
  {:else}
    <pre
      class="max-h-[28rem] overflow-auto rounded bg-background p-2 text-xs"><code
        >{content}</code
      ></pre>
  {/if}
</div>
