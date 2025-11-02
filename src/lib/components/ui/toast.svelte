<!--
	Production-ready Toast Notification Component
	Provides consistent toast notifications across the application
-->

<script lang="ts">
	import { cn } from '$lib/utils';
	import { Button } from './button';
	import { X, CheckCircle, AlertCircle, Info, AlertTriangle } from 'lucide-svelte';
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

	// Variant configurations
	const variantConfig = {
		default: {
			icon: Info,
			className: 'bg-background border-border text-foreground',
			iconClassName: 'text-foreground'
		},
		success: {
			icon: CheckCircle,
			className: 'bg-green-50 border-green-200 text-green-900 dark:bg-green-900/20 dark:border-green-800 dark:text-green-100',
			iconClassName: 'text-green-600 dark:text-green-400'
		},
		error: {
			icon: AlertCircle,
			className: 'bg-red-50 border-red-200 text-red-900 dark:bg-red-900/20 dark:border-red-800 dark:text-red-100',
			iconClassName: 'text-red-600 dark:text-red-400'
		},
		warning: {
			icon: AlertTriangle,
			className: 'bg-yellow-50 border-yellow-200 text-yellow-900 dark:bg-yellow-900/20 dark:border-yellow-800 dark:text-yellow-100',
			iconClassName: 'text-yellow-600 dark:text-yellow-400'
		},
		info: {
			icon: Info,
			className: 'bg-blue-50 border-blue-200 text-blue-900 dark:bg-blue-900/20 dark:border-blue-800 dark:text-blue-100',
			iconClassName: 'text-blue-600 dark:text-blue-400'
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
		'group pointer-events-auto relative flex w-full items-center justify-between space-x-4 overflow-hidden rounded-md border p-6 pr-8 shadow-lg transition-all',
		'data-[state=open]:animate-in data-[state=closed]:animate-out data-[swipe=end]:animate-out',
		'data-[state=closed]:fade-out-80 data-[state=closed]:slide-out-to-right-full data-[state=open]:slide-in-from-top-full data-[state=open]:sm:slide-in-from-bottom-full',
		config.className,
		className
	)}
	role="alert"
	aria-live="assertive"
	aria-atomic="true"
>
	<div class="flex items-start space-x-3">
		<Icon class={cn("h-5 w-5 flex-shrink-0", config.iconClassName)} />
		<div class="flex-1 space-y-1">
			{#if title}
				<div class="text-sm font-semibold">{title}</div>
			{/if}
			{#if description}
				<div class="text-sm opacity-90">{description}</div>
			{/if}
		</div>
	</div>

	{#if action}
		<Button
			variant="outline"
			size="sm"
			onclick={action.onClick}
			class="ml-4"
		>
			{action.label}
		</Button>
	{/if}

	<Button
		variant="ghost"
		size="sm"
		onclick={handleClose}
		class="absolute right-2 top-2 h-6 w-6 p-0 opacity-0 group-hover:opacity-100 transition-opacity"
		aria-label="Close notification"
	>
		<X class="h-4 w-4" />
	</Button>

	<!-- Progress bar -->
	{#if duration > 0}
		<div class="absolute bottom-0 left-0 h-1 bg-current opacity-20 w-full">
			<div 
				class="h-full bg-current transition-all duration-75 ease-linear"
				style="width: {progress}%"
			></div>
		</div>
	{/if}
</div>