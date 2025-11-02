<script lang="ts">
	export let title: string;
	export let status: string;
	export let progress: number = 0;
	export let showSpinner: boolean = true;
</script>

<div class="p-4 border rounded-lg bg-blue-50">
	<div class="flex items-center gap-4">
		{#if showSpinner}
			<div class="animate-spin rounded-full h-6 w-6 border-b-2 border-blue-600"></div>
		{/if}
		<div class="flex-1">
			<div class="font-medium">{title}</div>
			<div class="text-sm text-muted-foreground">{status}</div>
			<div class="w-full bg-gray-200 rounded-full h-2 mt-2">
				<div 
					class="bg-blue-600 h-2 rounded-full transition-all duration-300" 
					style="width: {Math.max(0, Math.min(100, progress))}%"
				></div>
			</div>
		</div>
	</div>
</div>
