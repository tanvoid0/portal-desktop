<!--
	Avatar Speech Bubble Component
	Displays suggestions from the avatar assistant
-->

<script lang="ts">
  import { X } from "@lucide/svelte";
  import { cn } from "$lib/utils";
  import { Button } from "$lib/components/ui/button";
  import type { AvatarSuggestion } from "$lib/domains/ai/types/avatar";

  interface Props {
    suggestion: AvatarSuggestion;
    onDismiss?: () => void;
    class?: string;
  }

  let { suggestion, onDismiss, class: className = "" }: Props = $props();

  let isVisible = $state(false);

  $effect(() => {
    // Animate in
    setTimeout(() => {
      isVisible = true;
    }, 10);
  });

  function handleDismiss() {
    isVisible = false;
    setTimeout(() => {
      onDismiss?.();
    }, 300);
  }
</script>

<div
  class={cn(
    "pointer-events-auto relative max-w-xs rounded-lg border bg-card p-4 shadow-lg transition-all duration-300",
    isVisible ? "translate-y-0 opacity-100" : "translate-y-2 opacity-0",
    className,
  )}
  role="alert"
  aria-live="polite"
>
  <!-- Close button -->
  <Button
    variant="ghost"
    size="icon"
    class="absolute right-1 top-1 h-6 w-6 rounded-full opacity-70 hover:opacity-100"
    onclick={handleDismiss}
    aria-label="Dismiss suggestion"
  >
    <X class="h-3 w-3" />
  </Button>

  <!-- Speech bubble tail -->
  <div
    class="absolute -bottom-2 left-8 h-4 w-4 rotate-45 border-b border-r bg-card"
  ></div>

  <!-- Content -->
  <div class="pr-6">
    <p class="text-sm text-card-foreground">{suggestion.message}</p>
  </div>
</div>
