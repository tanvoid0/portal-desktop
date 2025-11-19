<script lang="ts">
	import type { HTMLInputAttributes, HTMLInputTypeAttribute } from 'svelte/elements';
	import { cn, type WithElementRef } from '$lib/utils.js';

	type InputType = Exclude<HTMLInputTypeAttribute, 'file'>;

	type Props = WithElementRef<
		Omit<HTMLInputAttributes, 'type'> &
			({ type: 'file'; files?: FileList } | { type?: InputType; files?: undefined })
	> & {
		error?: string;
		required?: boolean;
		label?: string;
		description?: string;
	};

	let {
		ref = $bindable(null),
		value = $bindable(),
		type,
		files = $bindable(),
		class: className,
		error = '',
		required = false,
		label = '',
		description = '',
		...restProps
	}: Props = $props();


	// Memoize classes for performance
	const inputClasses = $derived(cn(
		'flex h-9 w-full min-w-0 rounded-md border border-input bg-background px-3 py-1 text-base shadow-xs ring-offset-background transition-[color,box-shadow] outline-none selection:bg-primary selection:text-primary-foreground placeholder:text-muted-foreground disabled:cursor-not-allowed disabled:opacity-50 md:text-sm dark:bg-input/30',
		'focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50',
		'aria-invalid:border-destructive aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40',
		error && 'border-destructive focus:ring-destructive',
		className
	));

	// Handle focus events
	let isFocused = $state(false);
	
	function handleFocus() {
		isFocused = true;
	}

	function handleBlur() {
		isFocused = false;
	}
</script>

<div class="space-y-2">
	{#if label}
		<label for={restProps.id} class="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70">
			{label}
			{#if required}
				<span class="text-destructive ml-1">*</span>
			{/if}
		</label>
	{/if}

	{#if type === 'file'}
		<input
			bind:this={ref}
			data-slot="input"
			class={inputClasses}
			type="file"
			bind:files
			bind:value
			onfocus={handleFocus}
			onblur={handleBlur}
			aria-invalid={!!error}
			aria-describedby={error ? `${restProps.id}-error` : undefined}
			{...restProps}
		/>
	{:else}
		<input
			bind:this={ref}
			data-slot="input"
			class={inputClasses}
			{type}
			bind:value
			onfocus={handleFocus}
			onblur={handleBlur}
			aria-invalid={!!error}
			aria-describedby={error ? `${restProps.id}-error` : undefined}
			{...restProps}
		/>
	{/if}

	{#if description}
		<p class="text-sm text-muted-foreground">{description}</p>
	{/if}

	{#if error}
		<p id={`${restProps.id}-error`} class="text-sm text-destructive" role="alert">
			{error}
		</p>
	{/if}
</div>
