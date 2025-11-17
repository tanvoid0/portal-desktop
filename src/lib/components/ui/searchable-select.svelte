<script lang="ts">
	import { cn } from '$lib/utils.js';
	import * as Popover from './popover';
	import * as Command from './command';
	import { Button } from './button';
	import { Check, ChevronsUpDown } from 'lucide-svelte';

	interface Props {
		options: Array<{ value: string; label: string }>;
		value?: string;
		placeholder?: string;
		searchPlaceholder?: string;
		onValueChange?: (value: string) => void;
		disabled?: boolean;
		class?: string;
	}

	let {
		options = [],
		value = '',
		placeholder = 'Select option...',
		searchPlaceholder = 'Search...',
		onValueChange,
		disabled = false,
		class: className = ''
	}: Props = $props();

	type Option = { value: string; label: string };

	let open = $state(false);
	let search = $state('');

	function filterOptions(opts: Option[], searchQuery: string): Option[] {
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

	const selectedOption = $derived(
		options.find((opt) => opt.value === value) as Option | undefined
	);

	function handleSelect(optionValue: string) {
		onValueChange?.(optionValue);
		open = false;
		search = '';
	}
</script>

<div class={cn('relative', className)}>
	<Popover.Root bind:open={open}>
		<Popover.Trigger>
			<Button
				variant="outline"
				role="combobox"
				aria-expanded={open}
				{disabled}
				class={cn(
					'w-full justify-between h-10 px-3 py-2 text-sm font-normal',
					!selectedOption && 'text-muted-foreground',
					open && 'ring-2 ring-ring ring-offset-2'
				)}
			>
				<span class="truncate text-left">
					{selectedOption?.label ?? placeholder}
				</span>
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
					<Command.Group>
						{#each filteredOptions as option}
							<Command.Item
								value={option.value}
								onclick={() => handleSelect(option.value)}
								class={cn(
									'relative flex cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none transition-colors',
									'aria-selected:bg-accent aria-selected:text-accent-foreground',
									'hover:bg-accent hover:text-accent-foreground',
									value === option.value && 'bg-accent/50 font-medium'
								)}
							>
								<Check
									class={cn(
										'mr-2 h-4 w-4 shrink-0 transition-opacity',
										value === option.value ? 'opacity-100 text-primary' : 'opacity-0'
									)}
								/>
								<span class="flex-1 truncate">{option.label}</span>
							</Command.Item>
						{/each}
					</Command.Group>
				</Command.List>
			</Command.Root>
		</Popover.Content>
	</Popover.Root>
</div>

