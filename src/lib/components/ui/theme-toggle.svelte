<script lang="ts">
	import { Button } from './button';
	import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuTrigger } from './dropdown-menu';
	import { themeStore, resolvedTheme } from '@/lib/domains/shared';
	import { Sun, Moon, Monitor } from 'lucide-svelte';

	let isOpen = false;

	function setTheme(theme: 'light' | 'dark' | 'system') {
		themeStore.setTheme(theme);
		isOpen = false;
	}
</script>

<DropdownMenu bind:open={isOpen}>
	<DropdownMenuTrigger>
		<Button
			variant="ghost"
			size="sm"
			class="h-8 w-8 px-0"
		>
			{#if $resolvedTheme === 'light'}
				<Sun class="h-4 w-4" />
			{:else}
				<Moon class="h-4 w-4" />
			{/if}
			<span class="sr-only">Toggle theme</span>
		</Button>
	</DropdownMenuTrigger>
	<DropdownMenuContent align="end">
		<DropdownMenuItem onclick={() => setTheme('light')}>
			<Sun class="mr-2 h-4 w-4" />
			<span>Light</span>
		</DropdownMenuItem>
		<DropdownMenuItem onclick={() => setTheme('dark')}>
			<Moon class="mr-2 h-4 w-4" />
			<span>Dark</span>
		</DropdownMenuItem>
		<DropdownMenuItem onclick={() => setTheme('system')}>
			<Monitor class="mr-2 h-4 w-4" />
			<span>System</span>
		</DropdownMenuItem>
	</DropdownMenuContent>
</DropdownMenu>
