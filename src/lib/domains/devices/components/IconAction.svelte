<script lang="ts">
  import type { Snippet } from 'svelte';
  import { Button } from '$lib/components/ui/button';
  import * as Tooltip from '$lib/components/ui/tooltip';

  interface Props {
    /** Tooltip text and accessible name — the button shows the icon only. */
    label: string;
    icon: Snippet;
    onclick: () => void;
    disabled?: boolean;
    variant?: 'ghost' | 'outline' | 'default';
    class?: string;
  }

  let {
    label,
    icon,
    onclick,
    disabled = false,
    variant = 'ghost',
    class: className = '',
  }: Props = $props();
</script>

<Tooltip.Provider delayDuration={200}>
  <Tooltip.Root>
    <Tooltip.Trigger>
      {#snippet child({ props })}
        <Button
          {...props}
          {variant}
          size="icon"
          class="h-7 w-7 {className}"
          {onclick}
          {disabled}
          aria-label={label}
        >
          {@render icon()}
        </Button>
      {/snippet}
    </Tooltip.Trigger>
    <Tooltip.Content>{label}</Tooltip.Content>
  </Tooltip.Root>
</Tooltip.Provider>
