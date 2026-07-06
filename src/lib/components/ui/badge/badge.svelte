<script lang="ts" module>
  import { type VariantProps, tv } from "tailwind-variants";

  export const badgeVariants = tv({
    base: "focus-visible:border-ring focus-visible:ring-ring/50 aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive inline-flex w-fit shrink-0 items-center justify-center gap-1.5 overflow-hidden whitespace-nowrap rounded-full border px-3 py-1 text-xs font-semibold transition-shadow duration-200 focus-visible:ring-[3px] [&>svg]:pointer-events-none [&>svg]:size-3 shadow-sm hover:shadow-md",
    variants: {
      variant: {
        default:
          "bg-gradient-to-r from-primary to-primary/90 text-primary-foreground [a&]:hover:from-primary/95 [a&]:hover:to-primary border-transparent",
        secondary:
          "bg-gradient-to-r from-secondary to-secondary/90 text-secondary-foreground [a&]:hover:from-secondary/95 [a&]:hover:to-secondary border-transparent",
        destructive:
          "bg-gradient-to-r from-destructive to-destructive/90 [a&]:hover:from-destructive/95 [a&]:hover:to-destructive focus-visible:ring-destructive/20 dark:focus-visible:ring-destructive/40 border-transparent text-white",
        outline:
          "text-foreground bg-background/60 backdrop-blur-sm border-2 border-border/50 [a&]:hover:bg-accent/60 [a&]:hover:border-primary/50 [a&]:hover:text-accent-foreground",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  });

  export type BadgeVariant = VariantProps<typeof badgeVariants>["variant"];
</script>

<script lang="ts">
  import type { HTMLAnchorAttributes } from "svelte/elements";
  import { cn, type WithElementRef } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    href,
    class: className,
    variant = "default",
    children,
    ...restProps
  }: WithElementRef<HTMLAnchorAttributes> & {
    variant?: BadgeVariant;
  } = $props();
</script>

<svelte:element
  this={href ? "a" : "span"}
  bind:this={ref}
  data-slot="badge"
  {href}
  class={cn(badgeVariants({ variant }), className)}
  {...restProps}
>
  {@render children?.()}
</svelte:element>
