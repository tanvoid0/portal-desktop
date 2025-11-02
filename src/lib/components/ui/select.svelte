<!--
	Select Component - Supports Four Input Formats:
	
	1. String Array: ['option1', 'option2', 'option3']
	   - Value and label are the same
	   - Good for simple cases
	
	2. Value-Label Object Array: [{ value: 'val', label: 'Label' }]
	   - Custom labels, disabled options supported
	   - Good for complex cases with custom formatting
	
	3. Constant Arrays: TASK_STATUS_OPTIONS (readonly arrays)
	   - Reusable across components
	   - Type-safe with TypeScript
	   - Good for shared options with custom labels
	
	4. TypeScript Enums: TaskStatusEnum
	   - Auto-generates human-readable labels from enum keys
	   - Full TypeScript enum support
	   - Converts SNAKE_CASE to "Snake Case"
	
	Examples:
	- Simple: options={['daily', 'weekly', 'monthly']}
	- Custom: options={[{ value: 'custom', label: 'Custom Option' }]}
	- Constants: options={TASK_STATUS_OPTIONS}
	- Enum: options={TaskStatusEnum}
-->
<script lang="ts">
	import { cn } from '@/lib/utils';
	import { onMount } from 'svelte';

	interface SelectOption {
		value: string;
		label: string;
		disabled?: boolean;
	}

	// Support for constant arrays (readonly arrays)
	type ConstantArray = readonly { readonly value: string; readonly label: string; readonly disabled?: boolean }[];
	
	// Support for TypeScript enums
	type EnumType = Record<string, string | number>;

	interface Props {
		options: string[] | SelectOption[] | ConstantArray | EnumType;
		defaultValue?: string;
		placeholder?: string;
		onSelect?: (value: string) => void;
		disabled?: boolean;
		required?: boolean;
		error?: string;
		class?: string;
	}

	let { 
		options = [], 
		defaultValue = '', 
		placeholder = 'Select an option...',
		onSelect = () => {},
		disabled = false,
		required = false,
		error = '',
		class: className = ''
	}: Props = $props();

	let selectedValue = $state(defaultValue || '');
	let isOpen = $state(false);
	let containerElement: HTMLDivElement;

	// Convert options to consistent format with memoization
	// Supports: string[], SelectOption[], constant arrays, or TypeScript enums
	let normalizedOptions = $derived(() => {
		// Handle TypeScript enums (objects with string/number values)
		if (typeof options === 'object' && !Array.isArray(options)) {
			return Object.entries(options).map(([key, value]) => ({
				value: String(value),
				label: key.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase()),
				disabled: false
			}));
		}
		
		// Handle arrays (string[], SelectOption[], constant arrays)
		return options.map(option => 
			typeof option === 'string' 
				? { value: option, label: option, disabled: false }
				: { disabled: false, ...option }
		);
	});

	// Memoize display value for performance
	let displayValue = $derived(() => {
		if (!selectedValue) return placeholder;
		const option = normalizedOptions().find(opt => opt.value === selectedValue);
		return option ? option.label : selectedValue;
	});

	function handleSelect(value: string) {
		selectedValue = value;
		onSelect(value);
		isOpen = false;
	}

	function toggleOpen() {
		if (!disabled) {
			isOpen = !isOpen;
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			isOpen = false;
		} else if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			toggleOpen();
		} else if (event.key === 'ArrowDown') {
			event.preventDefault();
			if (!isOpen) {
				isOpen = true;
			}
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			if (!isOpen) {
				isOpen = true;
			}
		}
	}

	// Close dropdown when clicking outside
	function handleClickOutside(event: MouseEvent) {
		if (containerElement && !containerElement.contains(event.target as Node)) {
			isOpen = false;
		}
	}

	// Add event listeners
	onMount(() => {
		document.addEventListener('click', handleClickOutside);
		return () => {
			document.removeEventListener('click', handleClickOutside);
		};
	});
</script>

<div 
	bind:this={containerElement}
	class={cn('relative', className)}
	onkeydown={handleKeydown}
	role="combobox"
	aria-expanded={isOpen}
	aria-haspopup="listbox"
	aria-controls="select-dropdown"
	aria-required={required}
	aria-invalid={!!error}
	tabindex="0"
>
	<!-- Trigger Button -->
	<button
		type="button"
		aria-expanded={isOpen}
		aria-haspopup="listbox"
		{disabled}
		onclick={toggleOpen}
		class={cn(
			'flex h-10 w-full items-center justify-between rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 transition-colors duration-200',
			error && 'border-destructive focus:ring-destructive',
			className
		)}
	>
		<span class={cn('block truncate', selectedValue ? 'text-foreground' : 'text-muted-foreground')}>
			{displayValue()}
		</span>
		<svg
			class={cn('h-4 w-4 opacity-50 transition-transform', isOpen && 'rotate-180')}
			fill="none"
			stroke="currentColor"
			viewBox="0 0 24 24"
		>
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
		</svg>
	</button>

	<!-- Dropdown Content -->
	{#if isOpen}
		<div
			id="select-dropdown"
			class="absolute top-full z-50 w-full rounded-md border bg-popover p-1 text-popover-foreground shadow-lg animate-in fade-in-0 zoom-in-95"
			role="listbox"
		>
			{#each normalizedOptions() as option (option.value)}
				<button
					type="button"
					role="option"
					aria-selected={selectedValue === option.value}
					disabled={option.disabled}
					onclick={() => !option.disabled && handleSelect(option.value)}
					class={cn(
						'relative flex w-full cursor-default select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm outline-none focus:bg-accent focus:text-accent-foreground disabled:pointer-events-none disabled:opacity-50',
						selectedValue === option.value && 'bg-accent text-accent-foreground',
						option.disabled && 'opacity-50 cursor-not-allowed'
					)}
				>
					{#if selectedValue === option.value}
						<svg
							class="absolute left-2 h-4 w-4"
							fill="none"
							stroke="currentColor"
							viewBox="0 0 24 24"
						>
							<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
						</svg>
					{/if}
					{option.label}
				</button>
			{/each}
		</div>
	{/if}
	
	<!-- Error Message -->
	{#if error}
		<p class="mt-1 text-sm text-destructive" role="alert">
			{error}
		</p>
	{/if}
</div>