<!-- Keyboard Shortcuts Help Modal -->
<script lang="ts">
	import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/lib/components/ui/dialog';
	import { NAVIGATION_SHORTCUTS, ACTION_SHORTCUTS, RESOURCE_TYPE_SHORTCUTS } from '../utils/keyboardConstants';
	import { formatShortcut, parseShortcut } from '../utils/shortcutParser';
	
	interface ShortcutGroup {
		title: string;
		shortcuts: Array<{ key: string; description: string }>;
	}
	
	interface Props {
		open?: boolean;
		onOpenChange?: (open: boolean) => void;
		context?: 'table' | 'detail' | 'overview';
	}
	
	let { open = $bindable(false), onOpenChange, context = 'table' }: Props = $props();
	
	const shortcutGroups = $derived.by<ShortcutGroup[]>(() => {
		const groups: ShortcutGroup[] = [
			{
				title: 'Navigation',
				shortcuts: [
					{ key: 'ArrowUp / k', description: 'Move up' },
					{ key: 'ArrowDown / j', description: 'Move down' },
					{ key: 'Enter', description: 'Open selected resource' },
					{ key: 'g', description: 'Go to top' },
					{ key: 'Shift+G', description: 'Go to bottom' },
					{ key: formatShortcut(NAVIGATION_SHORTCUTS.COMMAND_PALETTE[0]), description: 'Open command palette' },
					{ key: NAVIGATION_SHORTCUTS.HELP, description: 'Show this help' }
				]
			},
			{
				title: 'Resource Types',
				shortcuts: [
					{ key: RESOURCE_TYPE_SHORTCUTS.OVERVIEW, description: 'Overview' },
					{ key: RESOURCE_TYPE_SHORTCUTS.PODS, description: 'Pods' },
					{ key: RESOURCE_TYPE_SHORTCUTS.SERVICES, description: 'Services' },
					{ key: RESOURCE_TYPE_SHORTCUTS.DEPLOYMENTS, description: 'Deployments' },
					{ key: RESOURCE_TYPE_SHORTCUTS.STATEFULSETS, description: 'StatefulSets' },
					{ key: RESOURCE_TYPE_SHORTCUTS.DAEMONSETS, description: 'DaemonSets' },
					{ key: RESOURCE_TYPE_SHORTCUTS.JOBS, description: 'Jobs' },
					{ key: RESOURCE_TYPE_SHORTCUTS.CRONJOBS, description: 'CronJobs' },
					{ key: RESOURCE_TYPE_SHORTCUTS.CONFIGMAPS, description: 'ConfigMaps' },
					{ key: RESOURCE_TYPE_SHORTCUTS.SECRETS, description: 'Secrets' }
				]
			}
		];
		
		if (context === 'table') {
			groups.push({
				title: 'Actions',
				shortcuts: [
					{ key: ACTION_SHORTCUTS.DESCRIBE, description: 'Describe/View details' },
					{ key: ACTION_SHORTCUTS.EDIT, description: 'Edit YAML' },
					{ key: ACTION_SHORTCUTS.LOGS, description: 'View logs' },
					{ key: ACTION_SHORTCUTS.RESTART, description: 'Restart' },
					{ key: ACTION_SHORTCUTS.SCALE, description: 'Scale (deployments)' },
					{ key: ACTION_SHORTCUTS.YAML, description: 'View YAML' },
					{ key: `Ctrl+${ACTION_SHORTCUTS.DELETE}`, description: 'Delete' },
					{ key: ACTION_SHORTCUTS.PORT_FORWARD, description: 'Port forward' }
				]
			});
		}
		
		groups.push({
			title: 'Namespace',
			shortcuts: [
				{ key: NAVIGATION_SHORTCUTS.NAMESPACE_SWITCH[0], description: 'Switch namespace' },
				{ key: '0-9', description: 'Quick select namespace' }
			]
		});
		
		return groups;
	});
	
	function handleOpenChange(newOpen: boolean) {
		open = newOpen;
		onOpenChange?.(newOpen);
	}
</script>

<Dialog bind:open onOpenChange={handleOpenChange}>
	<DialogContent class="sm:max-w-[600px]">
		<DialogHeader>
			<DialogTitle>Keyboard Shortcuts</DialogTitle>
		</DialogHeader>
		
		<div class="space-y-6 py-4">
			{#each shortcutGroups as group}
				<div>
					<h3 class="text-sm font-semibold mb-2">{group.title}</h3>
					<div class="space-y-1">
						{#each group.shortcuts as shortcut}
							<div class="flex items-center justify-between py-1">
								<span class="text-sm text-muted-foreground">{shortcut.description}</span>
								<kbd class="px-2 py-1 text-xs font-semibold bg-muted rounded border">
									{shortcut.key}
								</kbd>
							</div>
						{/each}
					</div>
				</div>
			{/each}
		</div>
	</DialogContent>
</Dialog>

