<!--
	Production-ready Toast Notification Component
	Provides consistent toast notifications across the application
-->

<script lang="ts">
  import { cn } from "$lib/utils";
  import { Button } from "./button";
  import {
    X,
    CheckCircle,
    AlertCircle,
    Info,
    AlertTriangle,
  } from "@lucide/svelte";
  import { onMount } from "svelte";

  interface ToastProps {
    id: string;
    title?: string;
    description?: string;
    variant?: "default" | "success" | "error" | "warning" | "info";
    duration?: number;
    action?: {
      label: string;
      onClick: () => void;
    };
    onClose?: () => void;
    class?: string;
  }

  let {
    title,
    description,
    variant = "default",
    duration = 5000,
    action,
    onClose,
    class: className = "",
  }: ToastProps = $props();
  let isRemoving = $state(false);
  let isVisible = $state(false);

  // Variant configurations - matching alert styling exactly
  const variantConfig = {
    default: {
      icon: Info,
      className: "bg-card border-border text-card-foreground",
      iconClassName: "text-muted-foreground",
    },
    success: {
      icon: CheckCircle,
      className: "bg-card border-border text-card-foreground",
      iconClassName: "text-green-600 dark:text-green-400",
    },
    error: {
      icon: AlertCircle,
      className: "bg-card border-border text-card-foreground",
      iconClassName: "text-destructive",
    },
    warning: {
      icon: AlertTriangle,
      className: "bg-card border-border text-card-foreground",
      iconClassName: "text-yellow-600 dark:text-yellow-400",
    },
    info: {
      icon: Info,
      className: "bg-card border-border text-card-foreground",
      iconClassName: "text-primary",
    },
  };

  const config = variantConfig[variant];
  const Icon = config.icon;

  // Auto-dismiss functionality
  let timeoutId: ReturnType<typeof setTimeout>;

  function startTimer() {
    if (duration > 0) {
      timeoutId = setTimeout(() => {
        dismiss();
      }, duration);
    }
  }

  function pauseTimer() {
    if (timeoutId) {
      clearTimeout(timeoutId);
    }
  }

  function dismiss() {
    isRemoving = true;
    setTimeout(() => {
      onClose?.();
    }, 300);
  }

  function handleClose() {
    pauseTimer();
    dismiss();
  }

  onMount(() => {
    // Show toast with animation
    setTimeout(() => {
      isVisible = true;
    }, 10);

    // Start auto-dismiss
    startTimer();
  });
</script>

<div
  class={cn(
    "group pointer-events-auto relative flex w-full items-start gap-3 overflow-hidden rounded-lg border px-4 py-3 transition-all",
    isVisible ? "translate-x-0 opacity-100" : "translate-x-full opacity-0",
    isRemoving ? "translate-x-full opacity-0" : "",
    config.className,
    className,
  )}
  role="alert"
  aria-live="assertive"
  aria-atomic="true"
>
  <Icon
    class={cn("h-4 w-4 flex-shrink-0 translate-y-0.5", config.iconClassName)}
  />
  <div class="min-w-0 flex-1 space-y-0.5">
    {#if title}
      <div class="text-sm leading-none">{title}</div>
    {/if}
    {#if description}
      <div class="text-sm leading-relaxed text-muted-foreground">
        {description}
      </div>
    {/if}
  </div>

  {#if action}
    <Button
      variant="outline"
      size="sm"
      onclick={action.onClick}
      class="ml-2 shrink-0"
    >
      {action.label}
    </Button>
  {/if}

  <Button
    variant="ghost"
    size="sm"
    onclick={handleClose}
    class="-mr-1 h-6 w-6 shrink-0 p-0 opacity-0 transition-opacity group-hover:opacity-100"
    aria-label="Close notification"
  >
    <X class="h-3.5 w-3.5" />
  </Button>

  <!-- Progress bar — CSS-driven so it costs no main-thread work and can't
       outlive the component the way a setInterval handle could. -->
  {#if duration > 0}
    <div
      class="absolute bottom-0 left-0 h-[2px] w-full overflow-hidden bg-border"
    >
      <div
        class="toast-progress h-full w-full bg-muted-foreground/30"
        style="animation-duration: {duration}ms"
      ></div>
    </div>
  {/if}
</div>

<style>
  .toast-progress {
    transform-origin: left;
    animation-name: toast-progress;
    animation-timing-function: linear;
    animation-fill-mode: forwards;
  }

  @keyframes toast-progress {
    from {
      transform: scaleX(1);
    }
    to {
      transform: scaleX(0);
    }
  }

  @media (prefers-reduced-motion: reduce) {
    .toast-progress {
      animation: none;
    }
  }
</style>
