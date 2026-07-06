<script lang="ts">
  import { Card } from "$lib/components/ui/card";
  import { Bot, User } from "@lucide/svelte";
  import { marked } from "marked";
  import type { ChatMessage as ChatMessageType } from "../../types/index.js";

  interface Props {
    message: ChatMessageType;
    showLoader?: boolean;
  }

  let { message, showLoader = false }: Props = $props();

  // Configure marked options
  marked.setOptions({
    breaks: true, // Convert line breaks to <br>
    gfm: true, // Enable GitHub Flavored Markdown
  });

  // Convert markdown to HTML
  function renderMarkdown(content: string): string {
    return marked.parse(content) as string;
  }

  const renderedContent = $derived(renderMarkdown(message.content));
</script>

<div
  class="flex flex-col {message.role === 'user' ? 'items-end' : 'items-start'}"
>
  <Card
    class="max-w-[85%] px-4 py-2 {message.role === 'user'
      ? 'bg-primary text-primary-foreground'
      : 'bg-muted'}"
  >
    <div class="flex items-start gap-2">
      {#if message.role === "assistant"}
        <Bot class="mt-0.5 h-4 w-4 shrink-0" />
      {:else}
        <User class="mt-0.5 h-4 w-4 shrink-0" />
      {/if}
      {#if showLoader && message.role === "assistant" && !message.content}
        <div class="flex gap-1">
          <span class="h-2 w-2 animate-pulse rounded-full bg-foreground/50"
          ></span>
          <span
            class="h-2 w-2 animate-pulse rounded-full bg-foreground/50"
            style="animation-delay: 0.2s"
          ></span>
          <span
            class="h-2 w-2 animate-pulse rounded-full bg-foreground/50"
            style="animation-delay: 0.4s"
          ></span>
        </div>
      {:else}
        <div
          class="prose prose-sm dark:prose-invert max-w-none flex-1 text-sm {message.role ===
          'user'
            ? 'prose-invert'
            : ''} 
						prose-headings:mt-2 prose-headings:mb-1 prose-p:my-1 prose-ul:my-1 prose-ol:my-1
						prose-code:text-xs prose-pre:my-2 prose-pre:p-2 prose-pre:rounded prose-pre:overflow-x-auto
						prose-a:underline prose-strong:font-semibold prose-em:italic"
        >
          {@html renderedContent}
        </div>
      {/if}
    </div>
  </Card>
  {#if message.timestamp}
    <span class="mt-1 px-1 text-xs text-muted-foreground">
      {new Date(message.timestamp).toLocaleTimeString()}
    </span>
  {/if}
</div>
