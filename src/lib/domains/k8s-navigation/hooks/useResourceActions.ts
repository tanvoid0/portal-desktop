// Hook for resource action shortcuts

import { derived, get, type Readable } from 'svelte/store';
import { ACTION_SHORTCUTS } from '../utils/keyboardConstants';
import type { ResourceAction } from '../types';
import type { ICloudResource } from '$lib/domains/cloud/core/types';

export interface ResourceActionHandlers {
	onDescribe?: (resource: ICloudResource) => void;
	onEdit?: (resource: ICloudResource) => void;
	onLogs?: (resource: ICloudResource) => void;
	onRestart?: (resource: ICloudResource) => void;
	onScale?: (resource: ICloudResource) => void;
	onYaml?: (resource: ICloudResource) => void;
	onDelete?: (resource: ICloudResource) => void;
	onPortForward?: (resource: ICloudResource) => void;
	onRefresh?: () => void;
}

export interface UseResourceActionsOptions {
	resource?: ICloudResource | null;
	selectedIndex?: number | Readable<number>;
	resources?: ICloudResource[] | Readable<ICloudResource[]> | (() => ICloudResource[]);
	handlers: ResourceActionHandlers;
	enabled?: boolean;
}

export function useResourceActions(options: UseResourceActionsOptions) {
	const { resource, selectedIndex, resources, handlers, enabled = true } = options;
	
	// Normalize selectedIndex to a store
	const selectedIndexStore: Readable<number> = typeof selectedIndex === 'object' && 'subscribe' in selectedIndex
		? selectedIndex
		: { subscribe: (fn: (val: number) => void) => { fn(selectedIndex ?? -1); return () => {}; } };
	
	// Normalize resources to a store
	const resourcesStore: Readable<ICloudResource[]> = 
		typeof resources === 'object' && resources !== null && 'subscribe' in resources
			? resources as Readable<ICloudResource[]>
			: typeof resources === 'function'
			? { subscribe: (fn: (val: ICloudResource[]) => void) => { fn((resources as () => ICloudResource[])()); return () => {}; } }
			: { subscribe: (fn: (val: ICloudResource[]) => void) => { fn(resources ?? []); return () => {}; } };
	
	// Create a readable store for currentResource
	const currentResource: Readable<ICloudResource | null> = derived(
		[selectedIndexStore, resourcesStore],
		([$selectedIndex, $resources]) => {
			if (resource) return resource;
			if ($selectedIndex !== undefined && $resources && $selectedIndex >= 0 && $selectedIndex < $resources.length) {
				return $resources[$selectedIndex];
			}
			return null;
		}
	);
	
	// For reactive updates, compute actions based on currentResource
	const actions: Readable<ResourceAction[]> = derived(
		[currentResource],
		([$currentResource]) => {
			const res = $currentResource;
			if (!res) return [];
			
			const availableActions: ResourceAction[] = [];
			
			if (handlers.onDescribe) {
				availableActions.push({
					key: ACTION_SHORTCUTS.DESCRIBE,
					label: 'Describe',
					shortcut: ACTION_SHORTCUTS.DESCRIBE,
					action: () => handlers.onDescribe?.(res),
					enabled: () => true
				});
			}
			
			if (handlers.onEdit) {
				availableActions.push({
					key: ACTION_SHORTCUTS.EDIT,
					label: 'Edit',
					shortcut: ACTION_SHORTCUTS.EDIT,
					action: () => handlers.onEdit?.(res),
					enabled: () => true
				});
			}
			
			if (handlers.onLogs && (res.type === 'pod' || res.type === 'deployment')) {
				availableActions.push({
					key: ACTION_SHORTCUTS.LOGS,
					label: 'Logs',
					shortcut: ACTION_SHORTCUTS.LOGS,
					action: () => handlers.onLogs?.(res),
					enabled: () => true
				});
			}
			
			if (handlers.onRestart && (res.type === 'deployment' || res.type === 'statefulset')) {
				availableActions.push({
					key: ACTION_SHORTCUTS.RESTART,
					label: 'Restart',
					shortcut: ACTION_SHORTCUTS.RESTART,
					action: () => handlers.onRestart?.(res),
					enabled: () => true
				});
			}
			
			if (handlers.onScale && res.type === 'deployment') {
				availableActions.push({
					key: ACTION_SHORTCUTS.SCALE,
					label: 'Scale',
					shortcut: ACTION_SHORTCUTS.SCALE,
					action: () => handlers.onScale?.(res),
					enabled: () => true
				});
			}
			
			if (handlers.onYaml) {
				availableActions.push({
					key: ACTION_SHORTCUTS.YAML,
					label: 'YAML',
					shortcut: ACTION_SHORTCUTS.YAML,
					action: () => handlers.onYaml?.(res),
					enabled: () => true
				});
			}
			
			if (handlers.onDelete) {
				availableActions.push({
					key: ACTION_SHORTCUTS.DELETE,
					label: 'Delete',
					shortcut: `Ctrl+${ACTION_SHORTCUTS.DELETE}`,
					action: () => handlers.onDelete?.(res),
					enabled: () => true
				});
			}
			
			if (handlers.onPortForward && res.type === 'pod') {
				availableActions.push({
					key: ACTION_SHORTCUTS.PORT_FORWARD,
					label: 'Port Forward',
					shortcut: ACTION_SHORTCUTS.PORT_FORWARD,
					action: () => handlers.onPortForward?.(res),
					enabled: () => true
				});
			}
			
			if (handlers.onRefresh) {
				availableActions.push({
					key: ACTION_SHORTCUTS.REFRESH,
					label: 'Refresh',
					shortcut: ACTION_SHORTCUTS.REFRESH,
					action: () => handlers.onRefresh?.(),
					enabled: () => true
				});
			}
			
			return availableActions;
		}
	);
	
	function handleKeydown(event: KeyboardEvent): boolean {
		if (!enabled) return false;
		
		// Ignore if typing in input/textarea
		const target = event.target as HTMLElement;
		if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') {
			return false;
		}
		
		const res = get(currentResource);
		if (!res) return false;
		
		const currentActions = get(actions);
		const key = event.key.toLowerCase();
		
		// Handle Ctrl+Delete or Delete
		if ((event.ctrlKey || event.metaKey) && event.key === ACTION_SHORTCUTS.DELETE) {
			const deleteAction = currentActions.find((a: ResourceAction) => a.key === ACTION_SHORTCUTS.DELETE);
			if (deleteAction && res) {
				event.preventDefault();
				deleteAction.action(res);
				return true;
			}
		}
		
		// Handle single key actions
		if (!event.ctrlKey && !event.metaKey && !event.altKey && !event.shiftKey) {
			const action = currentActions.find((a: ResourceAction) => a.shortcut.toLowerCase() === key);
			if (action && res && action.enabled?.(res) !== false) {
				event.preventDefault();
				action.action(res);
				return true;
			}
		}
		
		return false;
	}
	
	return {
		actions,
		currentResource,
		handleKeydown
	};
}
