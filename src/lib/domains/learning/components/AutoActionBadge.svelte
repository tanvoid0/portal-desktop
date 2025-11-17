<script lang="ts">
	interface Props {
		actionType: string;
		autonomous?: boolean;
		safetyLevel?: 'safe' | 'low-risk' | 'medium-risk' | 'high-risk';
		confidence?: number;
	}

	const { actionType, autonomous = false, safetyLevel = 'low-risk', confidence = 0 }: Props = $props();

	const getBadgeColor = () => {
		if (autonomous) {
			switch (safetyLevel) {
				case 'safe':
					return 'bg-green-500/20 text-green-600 dark:text-green-400 border-green-500/30';
				case 'low-risk':
					return 'bg-blue-500/20 text-blue-600 dark:text-blue-400 border-blue-500/30';
				case 'medium-risk':
					return 'bg-yellow-500/20 text-yellow-600 dark:text-yellow-400 border-yellow-500/30';
				case 'high-risk':
					return 'bg-red-500/20 text-red-600 dark:text-red-400 border-red-500/30';
				default:
					return 'bg-gray-500/20 text-gray-600 dark:text-gray-400 border-gray-500/30';
			}
		}
		return 'bg-gray-500/20 text-gray-600 dark:text-gray-400 border-gray-500/30';
	};

	const getBadgeIcon = () => {
		if (autonomous) {
			return '🤖';
		}
		return '⚡';
	};

	const getSafetyLabel = () => {
		if (!autonomous) return 'Manual';
		
		switch (safetyLevel) {
			case 'safe':
				return 'Auto (Safe)';
			case 'low-risk':
				return 'Auto (Low Risk)';
			case 'medium-risk':
				return 'Auto (Medium Risk)';
			case 'high-risk':
				return 'Auto (High Risk)';
			default:
				return 'Auto';
		}
	};
</script>

<div
	class="inline-flex items-center gap-1.5 px-2 py-1 rounded-md text-xs font-medium border {getBadgeColor()}"
	title="Action type: {actionType}, Confidence: {confidence > 0 ? (confidence * 100).toFixed(0) + '%' : 'N/A'}"
>
	<span class="text-sm">{getBadgeIcon()}</span>
	<span>{getSafetyLabel()}</span>
	{#if confidence > 0}
		<span class="opacity-60">({(confidence * 100).toFixed(0)}%)</span>
	{/if}
</div>

<style>
	/* Additional styling can be added here if needed */
</style>
