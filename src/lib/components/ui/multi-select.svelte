<!--
	Multi-Select Component
	Allows selecting multiple options with checkboxes
-->
<script lang="ts">
  import { cn } from '$lib/utils';
  import * as Popover from './popover';
  import * as Command from './command';
  import { Button } from './button';
  import ChevronsUpDown from '~icons/fa6-solid/up-down';
  import X from '~icons/fa6-solid/xmark';
  import { Checkbox } from './checkbox';
  import Icon from '@iconify/svelte';

  interface SelectOption {
    value: string;
    label: string;
    icon?: string;
    iconType?: string;
    disabled?: boolean;
  }

  interface Props {
    options: SelectOption[];
    value?: string[];
    placeholder?: string;
    searchPlaceholder?: string;
    onValueChange?: (value: string[]) => void;
    disabled?: boolean;
    class?: string;
    maxDisplay?: number; // Max number of selected items to show before "and X more"
  }

  let {
    options = [],
    value = $bindable([]),
    placeholder = 'Select options...',
    searchPlaceholder = 'Search...',
    onValueChange,
    disabled = false,
    class: className = '',
    maxDisplay = 2,
  }: Props = $props();

  let open = $state(false);
  let search = $state('');

  function filterOptions(opts: SelectOption[], searchQuery: string): SelectOption[] {
    if (!searchQuery.trim()) {
      return opts;
    }
    const query = searchQuery.toLowerCase();
    return opts.filter((option) => {
      return (
        option.label.toLowerCase().includes(query) || option.value.toLowerCase().includes(query)
      );
    });
  }

  const filteredOptions = $derived(filterOptions(options, search));

  const selectedOptions = $derived(options.filter((opt) => value.includes(opt.value)));

  const displayText = $derived.by(() => {
    if (selectedOptions.length === 0) {
      return placeholder;
    }
    if (selectedOptions.length <= maxDisplay) {
      return selectedOptions.map((opt) => opt.label).join(', ');
    }
    const displayed = selectedOptions
      .slice(0, maxDisplay)
      .map((opt) => opt.label)
      .join(', ');
    return `${displayed} and ${selectedOptions.length - maxDisplay} more`;
  });

  function handleToggle(optionValue: string) {
    const newValue = value.includes(optionValue)
      ? value.filter((v) => v !== optionValue)
      : [...value, optionValue];
    value = newValue;
    onValueChange?.(newValue);
  }

  function handleRemove(optionValue: string) {
    const newValue = value.filter((v) => v !== optionValue);
    value = newValue;
    onValueChange?.(newValue);
  }

  function clearAll() {
    value = [];
    onValueChange?.([]);
  }
</script>

<div class={cn('relative', className)}>
  <Popover.Root bind:open>
    <Popover.Trigger>
      {#snippet child({ props })}
        <Button
          {...props}
          variant="outline"
          role="combobox"
          aria-expanded={open}
          {disabled}
          size="sm"
          class={cn(
            'w-full justify-between',
            selectedOptions.length === 0 && 'text-muted-foreground',
            open && 'ring-2 ring-ring ring-offset-2'
          )}
        >
          <span class="truncate text-left">
            {selectedOptions.length === 0 ? placeholder : `${selectedOptions.length} selected`}
          </span>
          <ChevronsUpDown
            class={cn(
              'ml-2 h-4 w-4 shrink-0 opacity-50 transition-transform duration-200',
              open && 'rotate-180'
            )}
          />
        </Button>
      {/snippet}
    </Popover.Trigger>
    <Popover.Content
      class="w-56 max-w-[calc(100vw-2rem)] p-0 shadow-md sm:w-64 md:w-72"
      align="start"
      sideOffset={4}
    >
      <Command.Root shouldFilter={false} class="rounded-lg">
        <Command.Input placeholder={searchPlaceholder} bind:value={search} class="h-9" />
        <Command.Empty class="py-6 text-center text-sm text-muted-foreground">
          No results found.
        </Command.Empty>
        <Command.List class="max-h-[300px] overflow-y-auto">
          {#if selectedOptions.length > 0}
            <Command.Group>
              <Command.Item
                onclick={clearAll}
                class="cursor-pointer px-2 py-1.5 text-xs text-muted-foreground hover:text-foreground"
              >
                Clear all ({selectedOptions.length})
              </Command.Item>
            </Command.Group>
          {/if}
          <Command.Group>
            {#each filteredOptions as option}
              <Command.Item
                value={option.value}
                onclick={() => handleToggle(option.value)}
                class={cn(
                  'relative flex cursor-pointer items-center rounded-sm px-2 py-1.5 text-sm transition-colors outline-none select-none',
                  'hover:bg-accent hover:text-accent-foreground',
                  value.includes(option.value) && 'bg-accent/50'
                )}
              >
                <Checkbox
                  checked={value.includes(option.value)}
                  onCheckedChange={() => handleToggle(option.value)}
                  class="mr-2"
                />
                {#if option.icon && option.iconType === 'devicon'}
                  <span class="mr-2 text-lg" class:devicon={true} data-icon={option.icon}></span>
                {:else if option.icon}
                  <Icon icon={option.icon} class="mr-2 h-4 w-4" />
                {/if}
                <span class="flex-1 truncate">{option.label}</span>
              </Command.Item>
            {/each}
          </Command.Group>
        </Command.List>
      </Command.Root>
    </Popover.Content>
  </Popover.Root>
  {#if selectedOptions.length > 0}
    <div class="mt-1 flex flex-wrap items-center gap-1">
      {#each selectedOptions.slice(0, maxDisplay) as option}
        <span
          class="inline-flex items-center gap-1 rounded-md bg-primary/10 px-2 py-0.5 text-xs text-primary"
        >
          {option.label}
          <Button
            type="button"
            variant="ghost"
            size="icon-sm"
            aria-label="Remove {option.label}"
            onclick={() => handleRemove(option.value)}
            class="h-4 w-4 rounded-full p-0 hover:bg-primary/20"
          >
            <X class="h-3 w-3" />
          </Button>
        </span>
      {/each}
      {#if selectedOptions.length > maxDisplay}
        <span class="text-xs text-muted-foreground">
          and {selectedOptions.length - maxDisplay} more
        </span>
      {/if}
    </div>
  {/if}
</div>
