<script lang="ts">
  import { renderMarkdown } from "./renderMarkdown.js";

  interface Props {
    content: string;
    variant?: "user" | "assistant";
    isStreaming?: boolean;
    class?: string;
  }

  let {
    content,
    variant = "assistant",
    isStreaming = false,
    class: className = "",
  }: Props = $props();

  let container: HTMLDivElement | null = $state(null);

  const renderedContent = $derived(
    content
      ? renderMarkdown(content, { streaming: isStreaming && variant === "assistant" })
      : "",
  );

  function attachCopyHandlers(root: HTMLDivElement) {
    const buttons = root.querySelectorAll<HTMLButtonElement>(".chat-code-copy");
    const cleanups: Array<() => void> = [];

    buttons.forEach((button) => {
      const label = button.querySelector<HTMLElement>(".chat-code-copy-label");
      const code = button
        .closest(".chat-code-block")
        ?.querySelector<HTMLElement>("code");
      if (!code || !label) return;

      let resetTimer: ReturnType<typeof setTimeout> | undefined;

      const onClick = async () => {
        try {
          await navigator.clipboard.writeText(code.textContent ?? "");
          label.textContent = "Copied!";
          button.classList.add("chat-code-copy--copied");
          if (resetTimer) clearTimeout(resetTimer);
          resetTimer = setTimeout(() => {
            label.textContent = "Copy";
            button.classList.remove("chat-code-copy--copied");
          }, 1800);
        } catch {
          label.textContent = "Failed";
          if (resetTimer) clearTimeout(resetTimer);
          resetTimer = setTimeout(() => {
            label.textContent = "Copy";
          }, 1800);
        }
      };

      button.addEventListener("click", onClick);
      cleanups.push(() => {
        button.removeEventListener("click", onClick);
        if (resetTimer) clearTimeout(resetTimer);
      });
    });

    return () => cleanups.forEach((cleanup) => cleanup());
  }

  function attachPreviewToggleHandlers(root: HTMLDivElement) {
    const buttons = root.querySelectorAll<HTMLButtonElement>(
      ".chat-md-preview-toggle",
    );
    const cleanups: Array<() => void> = [];

    buttons.forEach((button) => {
      const preview = button.closest(".chat-md-preview");
      const body = preview?.querySelector<HTMLElement>(".chat-md-preview-body");
      const source = preview?.querySelector<HTMLElement>(".chat-md-preview-source");
      if (!body || !source) return;

      const onClick = () => {
        const showingSource = !source.hidden;
        if (showingSource) {
          source.hidden = true;
          body.hidden = false;
          button.textContent = "Show source";
          button.setAttribute("data-mode", "source");
        } else {
          body.hidden = true;
          source.hidden = false;
          button.textContent = "Show preview";
          button.setAttribute("data-mode", "preview");
        }
      };

      button.addEventListener("click", onClick);
      cleanups.push(() => button.removeEventListener("click", onClick));
    });

    return () => cleanups.forEach((cleanup) => cleanup());
  }

  $effect(() => {
    renderedContent;
    if (!container) return;
    const cleanupCopy = attachCopyHandlers(container);
    const cleanupPreview = attachPreviewToggleHandlers(container);
    return () => {
      cleanupCopy();
      cleanupPreview();
    };
  });
</script>

<div
  bind:this={container}
  class="chat-markdown prose prose-sm max-w-none text-sm {variant === 'user'
    ? 'chat-markdown--user prose-invert'
    : 'dark:prose-invert'} {className}"
>
  {@html renderedContent}
  {#if isStreaming && variant === "assistant"}
    <span
      class="ml-0.5 inline-block h-[1.1em] w-[2px] translate-y-[2px] animate-cursor-blink rounded-full bg-foreground/80 align-middle"
      aria-hidden="true"
    ></span>
  {/if}
</div>
