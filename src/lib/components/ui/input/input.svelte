<script lang="ts">
	import type { HTMLInputAttributes, HTMLInputTypeAttribute } from "svelte/elements";
	import { cn, type WithElementRef } from "$lib/utils.js";

	type InputType = Exclude<HTMLInputTypeAttribute, "file">;

	type Props = WithElementRef<
		Omit<HTMLInputAttributes, "type"> &
			({ type: "file"; files?: FileList } | { type?: InputType; files?: undefined })
	>;

	let {
		ref = $bindable(null),
		value = $bindable(),
		type,
		files = $bindable(),
		class: className,
		"data-slot": dataSlot = "input",
		...restProps
	}: Props = $props();
</script>

{#if type === "file"}
	<input
		bind:this={ref}
		data-slot={dataSlot}
		class={cn(
			"selection:bg-primary dark:bg-input/30 selection:text-primary-foreground border-input/50 ring-offset-background placeholder:text-muted-foreground/60 shadow-button flex h-10 w-full min-w-0 rounded-xl border-2 bg-background/80 backdrop-blur-sm px-4 pt-2.5 text-sm font-medium outline-none transition-all duration-300 disabled:cursor-not-allowed disabled:opacity-50",
			"focus-visible:border-primary focus-visible:ring-primary/30 focus-visible:ring-[3px] focus-visible:shadow-glow-primary",
			"hover:border-primary/50 hover:shadow-button-hover",
			"aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
			className
		)}
		type="file"
		bind:files
		bind:value
		{...restProps}
	/>
{:else}
	<input
		bind:this={ref}
		data-slot={dataSlot}
		class={cn(
			"border-input/50 bg-background/80 backdrop-blur-sm selection:bg-primary selection:text-primary-foreground ring-offset-background placeholder:text-muted-foreground/60 shadow-button flex h-10 w-full min-w-0 rounded-xl border-2 px-4 py-2.5 text-base outline-none transition-all duration-300 disabled:cursor-not-allowed disabled:opacity-50 md:text-sm",
			"focus-visible:border-primary focus-visible:ring-primary/30 focus-visible:ring-[3px] focus-visible:shadow-glow-primary focus-visible:bg-background",
			"hover:border-primary/50 hover:shadow-button-hover",
			"aria-invalid:ring-destructive/20 dark:aria-invalid:ring-destructive/40 aria-invalid:border-destructive",
			className
		)}
		{type}
		bind:value
		{...restProps}
	/>
{/if}
