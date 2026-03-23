<!--
	Floating Avatar Assistant Component
	Animated avatar that provides contextual help and suggestions
-->

<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { get } from 'svelte/store';
	import { goto } from '$app/navigation';
	import { avatarStore, avatarActions } from '$lib/domains/ai/stores/avatarStore';
	import { avatarService } from '$lib/domains/ai/services/avatarService';
	import AvatarSpeechBubble from './AvatarSpeechBubble.svelte';
	import { cn } from '$lib/utils';
	import type { AvatarState, AvatarExpression } from '$lib/domains/ai/types/avatar';
	import { settingsService } from '$lib/domains/settings/services/settingsService';

	let avatarState = $state<AvatarState>('idle');
	let avatarExpression = $state<AvatarExpression>('neutral');
	let currentSuggestion = $state(get(avatarStore).currentSuggestion);
	let position = $state(get(avatarStore).position);
	let isVisible = $state(true);
	let avatarEnabled = $state(true);
	
	// Local position tracking for smooth animation
	const initialPos = get(avatarStore).position;
	let currentX = $state(initialPos.x);
	let currentY = $state(initialPos.y);

	// Subscribe to avatar store
	let unsubscribe: (() => void) | undefined;

	onMount(async () => {
		// Check if avatar is enabled in settings
		try {
			const settings = await settingsService.getSettings();
			avatarEnabled = settings.app.avatarEnabled ?? true;
		} catch (error) {
			console.error('Failed to load avatar setting:', error);
			avatarEnabled = true; // Default to enabled
		}

		// Only initialize if enabled
		if (!avatarEnabled) {
			return;
		}

		// Subscribe to store updates
		unsubscribe = avatarStore.subscribe((store) => {
			avatarState = store.state;
			avatarExpression = store.expression;
			currentSuggestion = store.currentSuggestion;
			position = store.position;
			isVisible = store.isVisible;
		});

		// Initialize avatar service
		avatarService.init();

		// Set initial position to bottom right
		updatePosition();

		// Handle window resize
		if (typeof window !== 'undefined') {
			window.addEventListener('resize', handleResize);
		}
	});

	onDestroy(() => {
		if (unsubscribe) {
			unsubscribe();
		}
		if (typeof window !== 'undefined') {
			window.removeEventListener('resize', handleResize);
		}
		avatarService.destroy();
	});

	function handleResize() {
		updatePosition();
	}

	// Update position when window size changes or store position changes
	$effect(() => {
		const storePos = position;
		// Only update if position is significantly different (e.g., from window resize)
		const pos = getBottomRightPosition();
		if (Math.abs(storePos.x - pos.x) > 5 || Math.abs(storePos.y - pos.y) > 5) {
			updatePosition();
		}
	});

	// Avatar dimensions
	const AVATAR_SIZE = 64;
	const PADDING = 20; // Minimum padding from window edges
	const SPEECH_BUBBLE_HEIGHT = 150; // Approximate height when visible

	function getBottomRightPosition() {
		if (typeof window === 'undefined') {
			return { x: 1000 - AVATAR_SIZE - PADDING, y: 800 - AVATAR_SIZE - PADDING };
		}
		
		return {
			x: window.innerWidth - AVATAR_SIZE - PADDING,
			y: window.innerHeight - AVATAR_SIZE - PADDING
		};
	}

	function updatePosition() {
		const pos = getBottomRightPosition();
		currentX = pos.x;
		currentY = pos.y;
		avatarActions.setPosition({ x: currentX, y: currentY });
	}

	function getSpeechBubbleBounds() {
		if (typeof window === 'undefined') {
			return { x: 0, translateX: '-50%' };
		}

		const BUBBLE_WIDTH = 320; // max-w-xs = 320px
		const avatarCenterX = AVATAR_SIZE / 2;
		const bubbleLeft = currentX + avatarCenterX - BUBBLE_WIDTH / 2;
		const bubbleRight = bubbleLeft + BUBBLE_WIDTH;

		// Since avatar is in bottom right, bubble should align to the left of avatar
		// Check if bubble would go outside window
		if (bubbleRight > window.innerWidth - PADDING) {
			// Bubble would go off right edge, align to left of avatar
			return { x: -BUBBLE_WIDTH + avatarCenterX, translateX: '0%' };
		} else {
			// Bubble fits, center it above avatar
			return { x: avatarCenterX, translateX: '-50%' };
		}
	}


	function handleDismiss() {
		avatarActions.dismissSuggestion();
	}

	function handleClick() {
		// If there's a suggestion, dismiss it first
		if (currentSuggestion) {
			handleDismiss();
		}
		// Navigate to AI chat page
		goto('/ai/chat');
	}

	// Expression-based SVG paths
	const expressions: Record<AvatarExpression, { eyes: string; mouth: string }> = {
		neutral: {
			eyes: 'M 20 25 Q 20 20 25 20 Q 30 20 30 25',
			mouth: 'M 20 35 Q 25 40 30 35'
		},
		thinking: {
			eyes: 'M 20 25 Q 20 20 25 20 Q 30 20 30 25 M 20 25 Q 20 30 25 30 Q 30 30 30 25',
			mouth: 'M 20 35 Q 25 38 30 35'
		},
		happy: {
			eyes: 'M 20 25 Q 20 20 25 20 Q 30 20 30 25',
			mouth: 'M 20 35 Q 25 42 30 35'
		},
		concerned: {
			eyes: 'M 20 25 Q 20 22 25 22 Q 30 22 30 25',
			mouth: 'M 20 35 Q 25 32 30 35'
		}
	};

	const currentExpression = $derived(expressions[avatarExpression] || expressions.neutral);

	// Animation classes based on state
	const stateClasses = $derived({
		idle: 'animate-float',
		thinking: 'animate-pulse',
		suggesting: 'animate-bounce-subtle',
		error: 'animate-shake',
		success: 'animate-bounce-subtle'
	});
</script>

{#if avatarEnabled && isVisible}
	<div
		class={cn(
			'fixed z-50 cursor-pointer transition-all duration-500',
			stateClasses[avatarState]
		)}
		style="left: {currentX}px; top: {currentY}px;"
		onclick={handleClick}
		onkeydown={(e) => e.key === 'Enter' || e.key === ' ' ? handleClick() : null}
		role="button"
		aria-label="AI Assistant"
		tabindex="0"
	>
		<!-- Avatar SVG -->
		<div class="relative">
			<svg
				width="64"
				height="64"
				viewBox="0 0 50 50"
				class="drop-shadow-lg"
			>
				<!-- Head circle -->
				<circle
					cx="25"
					cy="25"
					r="20"
					fill="currentColor"
					class="text-primary"
				/>
				
				<!-- Eyes -->
				<path
					d={currentExpression.eyes}
					stroke="white"
					stroke-width="2"
					fill="none"
					stroke-linecap="round"
				/>
				
				<!-- Mouth -->
				<path
					d={currentExpression.mouth}
					stroke="white"
					stroke-width="2"
					fill="none"
					stroke-linecap="round"
				/>
			</svg>

			<!-- Speech bubble -->
			{#if currentSuggestion}
				{@const bubbleBounds = getSpeechBubbleBounds()}
				<div 
					class="absolute bottom-full mb-2"
					style="left: {bubbleBounds.x}px; transform: translateX({bubbleBounds.translateX});"
				>
					<AvatarSpeechBubble
						suggestion={currentSuggestion}
						onDismiss={handleDismiss}
					/>
				</div>
			{/if}
		</div>
	</div>
{/if}

<style>
	@keyframes float {
		0%, 100% {
			transform: translateY(0px);
		}
		50% {
			transform: translateY(-10px);
		}
	}

	@keyframes bounce-subtle {
		0%, 100% {
			transform: translateY(0px);
		}
		50% {
			transform: translateY(-5px);
		}
	}

	@keyframes shake {
		0%, 100% {
			transform: translateX(0);
		}
		25% {
			transform: translateX(-5px);
		}
		75% {
			transform: translateX(5px);
		}
	}

	.animate-float {
		animation: float 3s ease-in-out infinite;
	}

	.animate-bounce-subtle {
		animation: bounce-subtle 1s ease-in-out infinite;
	}

	.animate-shake {
		animation: shake 0.5s ease-in-out infinite;
	}
</style>

