<script lang="ts">
  import type { Snippet } from 'svelte';
  import * as ContextMenu from '$lib/components/ui/context-menu';
  import type { MenuItem } from './types';

  interface Props {
    /** Actions to show on right-click. Empty ⇒ no menu (trigger still renders). */
    items: MenuItem[];
    /**
     * The element the menu attaches to. Spread the given `props` onto your root
     * element (`<Card {...props}>`) so the right-click handler lands on it
     * directly — no wrapper element, so drag-and-drop and list dividers are
     * untouched.
     */
    trigger: Snippet<[Record<string, unknown>]>;
    /** Width class for the content panel. */
    class?: string;
  }

  let { items, trigger, class: className = 'w-52' }: Props = $props();
</script>

<ContextMenu.Root>
  <ContextMenu.Trigger>
    {#snippet child({ props })}
      {@render trigger(props)}
    {/snippet}
  </ContextMenu.Trigger>

  {#if items.length}
    <ContextMenu.Content class={className}>
      {#each items as item, i (i)}
        {#if item.separatorBefore && i > 0}
          <ContextMenu.Separator />
        {/if}

        {#if item.submenu?.length}
          <ContextMenu.Sub>
            <ContextMenu.SubTrigger>
              {#if item.icon}
                {@const Ico = item.icon}
                <Ico />
              {/if}
              {item.label}
            </ContextMenu.SubTrigger>
            <ContextMenu.SubContent class="w-44">
              {#each item.submenu as sub, j (j)}
                <ContextMenu.Item
                  disabled={sub.disabled}
                  variant={sub.variant ?? 'default'}
                  onSelect={sub.onSelect}
                >
                  {#if sub.icon}
                    {@const SubIco = sub.icon}
                    <SubIco />
                  {/if}
                  {sub.label}
                </ContextMenu.Item>
              {/each}
            </ContextMenu.SubContent>
          </ContextMenu.Sub>
        {:else}
          <ContextMenu.Item
            disabled={item.disabled}
            variant={item.variant ?? 'default'}
            onSelect={item.onSelect}
          >
            {#if item.icon}
              {@const Ico = item.icon}
              <Ico />
            {/if}
            {item.label}
          </ContextMenu.Item>
        {/if}
      {/each}
    </ContextMenu.Content>
  {/if}
</ContextMenu.Root>
