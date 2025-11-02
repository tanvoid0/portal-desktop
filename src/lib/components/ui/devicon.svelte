<!--
	Devicon Component - Wrapper for devicons with consistent styling
	Provides a clean interface for using devicons throughout the app
-->

<script lang="ts">
	interface Props {
		icon: string;
		size?: 'sm' | 'md' | 'lg' | 'xl' | '2xl' | '3xl' | '4xl';
		color?: string;
		class?: string;
	}

	const { icon, size = 'md', color, class: className = '' }: Props = $props();

	const sizeClasses = {
		sm: 'text-sm',
		md: 'text-base',
		lg: 'text-lg',
		xl: 'text-xl',
		'2xl': 'text-2xl',
		'3xl': 'text-3xl',
		'4xl': 'text-4xl'
	};

	const sizeClass = sizeClasses[size];
</script>

<i 
	class="devicon-icon {icon} {sizeClass} {className}"
	style={color ? `color: ${color}` : ''}
	aria-hidden="true"
></i>
