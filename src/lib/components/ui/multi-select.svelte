<!--
	Multi-Select Component
	Allows selecting multiple options with checkboxes
-->
<script lang="ts">
	import { cn } from '@/lib/utils';
	import * as Popover from './popover';
	import * as Command from './command';
	import { Button } from './button';
	import { ChevronsUpDown, X } from 'lucide-svelte';
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
		maxDisplay = 2
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
				option.label.toLowerCase().includes(query) ||
				option.value.toLowerCase().includes(query)
			);
		});
	}

	const filteredOptions = $derived(filterOptions(options, search));

	const selectedOptions = $derived(
		options.filter((opt) => value.includes(opt.value))
	);

	const displayText = $derived(() => {
		if (selectedOptions.length === 0) {
			return placeholder;
		}
		if (selectedOptions.length <= maxDisplay) {
			return selectedOptions.map(opt => opt.label).join(', ');
		}
		const displayed = selectedOptions.slice(0, maxDisplay).map(opt => opt.label).join(', ');
		return `${displayed} and ${selectedOptions.length - maxDisplay} more`;
	});

	function handleToggle(optionValue: string) {
		const newValue = value.includes(optionValue)
			? value.filter(v => v !== optionValue)
			: [...value, optionValue];
		value = newValue;
		onValueChange?.(newValue);
	}

	function handleRemove(optionValue: string, event: MouseEvent) {
		event.stopPropagation();
		const newValue = value.filter(v => v !== optionValue);
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
			<Button
				variant="outline"
				role="combobox"
				aria-expanded={open}
				{disabled}
				class={cn(
					'w-full justify-between h-auto min-h-10 px-3 py-2 text-sm font-normal',
					selectedOptions.length === 0 && 'text-muted-foreground',
					open && 'ring-2 ring-ring ring-offset-2'
				)}
			>
				<div class="flex-1 flex flex-wrap gap-1 items-center">
					{#if selectedOptions.length === 0}
						<span class="text-muted-foreground">{placeholder}</span>
					{:else}
						{#each selectedOptions.slice(0, maxDisplay) as option}
							<span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-md bg-primary/10 text-primary text-xs">
								{option.label}
								<button
									type="button"
									onclick={(e) => handleRemove(option.value, e)}
									class="hover:bg-primary/20 rounded-full p-0.5"
									tabindex="-1"
								>
									<X class="h-3 w-3" />
								</button>
							</span>
						{/each}
						{#if selectedOptions.length > maxDisplay}
							<span class="text-xs text-muted-foreground">
								and {selectedOptions.length - maxDisplay} more
							</span>
						{/if}
					{/if}
				</div>
				<ChevronsUpDown class={cn(
					'ml-2 h-4 w-4 shrink-0 opacity-50 transition-transform duration-200',
					open && 'rotate-180'
				)} />
			</Button>
		</Popover.Trigger>
		<Popover.Content 
			class="w-[var(--radix-popover-trigger-width)] p-0 shadow-md" 
			align="start"
			sideOffset={4}
		>
			<Command.Root shouldFilter={false} class="rounded-lg">
				<Command.Input 
					placeholder={searchPlaceholder} 
					bind:value={search}
					class="h-9"
				/>
				<Command.Empty class="py-6 text-center text-sm text-muted-foreground">
					No results found.
				</Command.Empty>
				<Command.List class="max-h-[300px] overflow-y-auto">
					{#if selectedOptions.length > 0}
						<Command.Group>
							<Command.Item
								onclick={clearAll}
								class="text-xs text-muted-foreground hover:text-foreground cursor-pointer px-2 py-1.5"
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
									'relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors',
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
</div>

