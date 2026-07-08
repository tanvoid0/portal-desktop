<script lang="ts">
  import ChatMarkdown from "./ChatMarkdown.svelte";

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
    <button
      type="button"
      class="rounded px-2 py-0.5 text-[10px] font-medium transition-colors {view === 'rendered'
        ? 'bg-primary/15 text-primary'
        : 'text-muted-foreground hover:text-foreground'}"
      onclick={() => (view = "rendered")}
    >
      Rendered
    </button>
    <button
      type="button"
      class="rounded px-2 py-0.5 text-[10px] font-medium transition-colors {view === 'source'
        ? 'bg-primary/15 text-primary'
        : 'text-muted-foreground hover:text-foreground'}"
      onclick={() => (view = "source")}
    >
      Source
    </button>
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
