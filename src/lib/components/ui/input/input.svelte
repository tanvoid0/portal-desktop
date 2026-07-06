<script lang="ts">
  import type {
    HTMLInputAttributes,
    HTMLInputTypeAttribute,
  } from "svelte/elements";
  import { cn, type WithElementRef } from "$lib/utils.js";

  type InputType = Exclude<HTMLInputTypeAttribute, "file">;

  type Props = WithElementRef<
    Omit<HTMLInputAttributes, "type"> &
      (
        | { type: "file"; files?: FileList }
        | { type?: InputType; files?: undefined }
      )
  >;

  let {
    ref = $bindable(null),
    value = $bindable(),
    type,
    files = $bindable(),
    class: className,
    "data-slot": dataSlot = "input",
    ...restProps
  }: Props = $props();
</script>

{#if type === "file"}
  <input
    bind:this={ref}
    data-slot={dataSlot}
    class={cn(
      "flex h-10 w-full min-w-0 rounded-xl border-2 border-input/50 bg-background/80 px-4 pt-2.5 text-sm font-medium shadow-button outline-none ring-offset-background backdrop-blur-sm transition-all duration-300 selection:bg-primary selection:text-primary-foreground placeholder:text-muted-foreground/60 disabled:cursor-not-allowed disabled:opacity-50 dark:bg-input/30",
      "focus-visible:border-primary focus-visible:shadow-glow-primary focus-visible:ring-[3px] focus-visible:ring-primary/30",
      "hover:border-primary/50 hover:shadow-button-hover",
      "aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
      className,
    )}
    type="file"
    bind:files
    bind:value
    {...restProps}
  />
{:else}
  <input
    bind:this={ref}
    data-slot={dataSlot}
    class={cn(
      "flex h-10 w-full min-w-0 rounded-xl border-2 border-input/50 bg-background/80 px-4 py-2.5 text-base shadow-button outline-none ring-offset-background backdrop-blur-sm transition-all duration-300 selection:bg-primary selection:text-primary-foreground placeholder:text-muted-foreground/60 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
      "focus-visible:border-primary focus-visible:bg-background focus-visible:shadow-glow-primary focus-visible:ring-[3px] focus-visible:ring-primary/30",
      "hover:border-primary/50 hover:shadow-button-hover",
      "aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
      className,
    )}
    {type}
    bind:value
    {...restProps}
  />
{/if}
