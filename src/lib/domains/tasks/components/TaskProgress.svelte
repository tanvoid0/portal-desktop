<script lang="ts">
	import { Card, CardContent } from '@/lib/components/ui/card';
	import { taskStats } from '../stores/taskStore';
</script>

<Card class="h-full">
	<CardContent class="p-3 h-full flex flex-col justify-center">
		<div class="space-y-2">
			<div class="flex items-center justify-between">
				<span class="text-xs font-medium text-foreground">Progress</span>
				<span class="text-xs font-medium text-foreground">{$taskStats.completionPercentage}%</span>
			</div>
			<div class="w-full bg-muted rounded-full h-2">
				<div 
					class="bg-gradient-to-r from-primary to-success-500 h-2 rounded-full transition-all duration-300" 
					style="width: {$taskStats.completionPercentage}%"
				></div>
			</div>
			<div class="text-xs text-muted-foreground text-center">
				{$taskStats.completed} of {$taskStats.total} completed
			</div>
		</div>
	</CardContent>
</Card>
