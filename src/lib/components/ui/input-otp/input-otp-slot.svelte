<script lang="ts">
  import { PinInput as InputOTPPrimitive } from "bits-ui";
  import { cn } from "$lib/utils.js";

  let {
    ref = $bindable(null),
    cell,
    class: className,
    ...restProps
  }: InputOTPPrimitive.CellProps = $props();
</script>

<InputOTPPrimitive.Cell
  {cell}
  bind:ref
  data-slot="input-otp-slot"
  class={cn(
    "aria-invalid:border-destructive relative flex size-9 items-center justify-center border-y border-r border-input text-sm outline-none transition-all first:rounded-l-md first:border-l last:rounded-r-md dark:bg-input/30",
    cell.isActive &&
      "aria-invalid:border-destructive dark:aria-invalid:ring-destructive/40 aria-invalid:ring-destructive/20 z-10 border-ring ring-[3px] ring-ring/50 ring-offset-background",
    className,
  )}
  {...restProps}
>
  {cell.char}
  {#if cell.hasFakeCaret}
    <div
      class="pointer-events-none absolute inset-0 flex items-center justify-center"
    >
      <div
        class="animate-caret-blink h-4 w-px bg-foreground duration-1000"
      ></div>
    </div>
  {/if}
</InputOTPPrimitive.Cell>
