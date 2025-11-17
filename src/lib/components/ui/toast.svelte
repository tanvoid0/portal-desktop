<!--
	Production-ready Toast Notification Component
	Provides consistent toast notifications across the application
-->

<script lang="ts">
	import { cn } from '$lib/utils';
	import { Button } from './button';
	import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from '@lucide/svelte';
	import { onMount } from 'svelte';

	interface ToastProps {
		id: string;
		title?: string;
		description?: string;
		variant?: 'default' | 'success' | 'error' | 'warning' | 'info';
		duration?: number;
		action?: {
			label: string;
			onClick: () => void;
		};
		onClose?: () => void;
		class?: string;
	}

	let {
		title,
		description,
		variant = 'default',
		duration = 5000,
		action,
		onClose,
		class: className = ''
	}: ToastProps = $props();
	let progress = $state(100);
	let isRemoving = $state(false);
	let isVisible = $state(false);

	// Variant configurations - matching alert styling exactly
	const variantConfig = {
		default: {
			icon: Info,
			className: 'bg-card border-border text-card-foreground',
			iconClassName: 'text-muted-foreground'
		},
		success: {
			icon: CheckCircle,
			className: 'bg-card border-border text-card-foreground',
			iconClassName: 'text-green-600 dark:text-green-400'
		},
		error: {
			icon: AlertCircle,
			className: 'bg-card border-border text-card-foreground',
			iconClassName: 'text-destructive'
		},
		warning: {
			icon: AlertTriangle,
			className: 'bg-card border-border text-card-foreground',
			iconClassName: 'text-yellow-600 dark:text-yellow-400'
		},
		info: {
			icon: Info,
			className: 'bg-card border-border text-card-foreground',
			iconClassName: 'text-primary'
		}
	};

	const config = variantConfig[variant];
	const Icon = config.icon;

	// Auto-dismiss functionality
	let timeoutId: ReturnType<typeof setTimeout>;

	function startTimer() {
		if (duration > 0) {
			timeoutId = setTimeout(() => {
				dismiss();
			}, duration);
		}
	}

	function pauseTimer() {
		if (timeoutId) {
			clearTimeout(timeoutId);
		}
	}


	function dismiss() {
		isRemoving = true;
		setTimeout(() => {
			onClose?.();
		}, 300);
	}

	function handleClose() {
		pauseTimer();
		dismiss();
	}

	// Progress bar animation
	function updateProgress() {
		if (duration > 0) {
			const startTime = Date.now();
			const updateInterval = setInterval(() => {
				const elapsed = Date.now() - startTime;
				const remaining = Math.max(0, duration - elapsed);
				progress = (remaining / duration) * 100;
				
				if (remaining <= 0) {
					clearInterval(updateInterval);
				}
			}, 50);
		}
	}

	onMount(() => {
		// Show toast with animation
		setTimeout(() => {
			isVisible = true;
		}, 10);

		// Start auto-dismiss
		startTimer();
		updateProgress();
	});
</script>

<div
	class={cn(
		'group pointer-events-auto relative flex w-full items-start gap-3 overflow-hidden rounded-lg border px-4 py-3 transition-all',
		isVisible ? 'opacity-100 translate-x-0' : 'opacity-0 translate-x-full',
		isRemoving ? 'opacity-0 translate-x-full' : '',
		config.className,
		className
	)}
	role="alert"
	aria-live="assertive"
	aria-atomic="true"
>
	<Icon class={cn("h-4 w-4 flex-shrink-0 translate-y-0.5", config.iconClassName)} />
	<div class="flex-1 space-y-0.5 min-w-0">
		{#if title}
			<div class="text-sm leading-none">{title}</div>
		{/if}
		{#if description}
			<div class="text-sm text-muted-foreground leading-relaxed">{description}</div>
		{/if}
	</div>

	{#if action}
		<Button
			variant="outline"
			size="sm"
			onclick={action.onClick}
			class="ml-2 shrink-0"
		>
			{action.label}
		</Button>
	{/if}

	<Button
		variant="ghost"
		size="sm"
		onclick={handleClose}
		class="h-6 w-6 p-0 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity -mr-1"
		aria-label="Close notification"
	>
		<X class="h-3.5 w-3.5" />
	</Button>

	<!-- Progress bar -->
	{#if duration > 0}
		<div class="absolute bottom-0 left-0 h-[2px] bg-border w-full overflow-hidden">
			<div 
				class="h-full bg-muted-foreground/30 transition-all duration-75 ease-linear"
				style="width: {progress}%"
			></div>
		</div>
	{/if}
</div>