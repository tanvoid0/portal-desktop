<script lang="ts" module>
  import { cn, type WithElementRef } from "$lib/utils.js";
  import type {
    HTMLAnchorAttributes,
    HTMLButtonAttributes,
  } from "svelte/elements";
  import { type VariantProps, tv } from "tailwind-variants";

  export const buttonVariants = tv({
    base: "focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive inline-flex shrink-0 items-center justify-center gap-2 whitespace-nowrap rounded-xl text-sm font-semibold outline-none transition-shadow duration-200 focus-visible:ring-[3px] disabled:pointer-events-none disabled:opacity-50 aria-disabled:pointer-events-none aria-disabled:opacity-50 [&_svg:not([class*='size-'])]:size-4 [&_svg]:pointer-events-none [&_svg]:shrink-0 relative overflow-hidden group",
    variants: {
      variant: {
        default:
          "bg-gradient-to-r from-primary via-primary/95 to-primary text-primary-foreground shadow-button hover:shadow-button-hover hover:from-primary/95 hover:via-primary hover:to-primary/95",
        destructive:
          "bg-gradient-to-r from-destructive via-destructive/95 to-destructive text-white shadow-button hover:shadow-button-hover focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40",
        outline:
          "bg-background/80 text-foreground backdrop-blur-sm shadow-button border-2 border-border/50 hover:bg-accent/50 hover:border-primary/50 hover:text-accent-foreground hover:shadow-button-hover dark:bg-input/20 dark:border-input/50 dark:hover:bg-input/40",
        secondary:
          "bg-gradient-to-r from-secondary via-secondary/95 to-secondary text-secondary-foreground shadow-button hover:shadow-button-hover hover:from-secondary/90 hover:via-secondary hover:to-secondary/90",
        ghost:
          "text-foreground hover:bg-accent/60 hover:text-accent-foreground dark:hover:bg-accent/40 rounded-lg",
        link: "text-primary underline-offset-4 hover:underline hover:text-primary/80",
      },
      size: {
        default: "h-10 px-5 py-2.5 has-[>svg]:px-4",
        sm: "h-8 gap-1.5 rounded-lg px-3.5 has-[>svg]:px-3 text-xs",
        lg: "h-12 rounded-xl px-7 has-[>svg]:px-5 text-base",
        icon: "size-10",
        "icon-sm": "size-8",
        "icon-lg": "size-12",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  });

  export type ButtonVariant = VariantProps<typeof buttonVariants>["variant"];
  export type ButtonSize = VariantProps<typeof buttonVariants>["size"];

  export type ButtonProps = WithElementRef<HTMLButtonAttributes> &
    WithElementRef<HTMLAnchorAttributes> & {
      variant?: ButtonVariant;
      size?: ButtonSize;
    };
</script>

<script lang="ts">
  let {
    class: className,
    variant = "default",
    size = "default",
    ref = $bindable(null),
    href = undefined,
    type = "button",
    disabled,
    children,
    ...restProps
  }: ButtonProps = $props();
</script>

{#if href}
  <a
    bind:this={ref}
    data-slot="button"
    class={cn(buttonVariants({ variant, size }), className)}
    href={disabled ? undefined : href}
    aria-disabled={disabled}
    role={disabled ? "link" : undefined}
    tabindex={disabled ? -1 : undefined}
    {...restProps}
  >
    {#if variant === "default" || variant === "destructive" || variant === "secondary"}
      <span
        class="absolute inset-0 bg-gradient-to-r from-white/15 via-transparent to-transparent opacity-0 transition-opacity duration-200 group-hover:opacity-100"
      ></span>
    {/if}
    <span class="relative z-10 inline-flex items-center gap-inherit"
      >{@render children?.()}</span
    >
  </a>
{:else}
  <button
    bind:this={ref}
    data-slot="button"
    class={cn(buttonVariants({ variant, size }), className)}
    {type}
    {disabled}
    {...restProps}
  >
    {#if variant === "default" || variant === "destructive" || variant === "secondary"}
      <span
        class="absolute inset-0 bg-gradient-to-r from-white/15 via-transparent to-transparent opacity-0 transition-opacity duration-200 group-hover:opacity-100"
      ></span>
    {/if}
    <span class="relative z-10 inline-flex items-center gap-inherit"
      >{@render children?.()}</span
    >
  </button>
{/if}
