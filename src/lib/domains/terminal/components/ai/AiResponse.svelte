<!--
  AiResponse — renders an AI answer with fenced code blocks as runnable
  command suggestions (Run inserts + executes, Copy copies).
-->
<script lang="ts">
  import { Check, Copy, Play } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { parseAiResponse } from "../../services/terminalAiContext";

  interface Props {
    content: string;
    onRunCommand?: (command: string) => void;
  }

  let { content, onRunCommand }: Props = $props();

  const segments = $derived(parseAiResponse(content));
  let copiedIdx = $state<number | null>(null);

  async function copy(text: string, idx: number) {
    try {
      await navigator.clipboard.writeText(text);
      copiedIdx = idx;
      setTimeout(() => (copiedIdx = null), 1500);
    } catch {
      // ignore
    }
  }
</script>

<div class="space-y-2">
  {#each segments as seg, i}
    {#if seg.type === "text"}
      <p class="whitespace-pre-wrap text-sm leading-relaxed text-foreground/90">
        {seg.content}
      </p>
    {:else}
      <div class="chat-code-block">
        <div class="chat-code-header">
          <span class="chat-code-lang">
            {seg.language || "command"}
          </span>
          <div class="flex items-center gap-0.5">
            <Button
              variant="ghost"
              size="sm"
              class="h-5 w-5 p-0"
              title="Copy"
              onclick={() => copy(seg.content, i)}
            >
              {#if copiedIdx === i}<Check class="h-3 w-3 text-status-success" />{:else}<Copy class="h-3 w-3" />{/if}
            </Button>
            {#if onRunCommand}
              <Button
                variant="ghost"
                size="sm"
                class="h-5 gap-1 px-1.5 text-[10px] text-status-success"
                title="Run in terminal"
                onclick={() => onRunCommand?.(seg.content)}
              >
                <Play class="h-3 w-3" />
                Run
              </Button>
            {/if}
          </div>
        </div>
        <pre class="chat-code-pre"><code>{seg.content}</code></pre>
      </div>
    {/if}
  {/each}
</div>
