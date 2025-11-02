/**
 * Breadcrumb store for managing navigation state
 * Provides centralized breadcrumb management across the application
 */

import { writable, derived } from 'svelte/store';
import { page } from '$app/stores';
import { logger } from '../services/logger';

const log = logger.createScoped('BreadcrumbStore');

export interface BreadcrumbItem {
	label: string;
	href?: string;
	icon?: string;
	disabled?: boolean;
}

// Store for custom breadcrumb items (set by pages)
const customBreadcrumbs = writable<BreadcrumbItem[]>([]);

// Store for page-specific breadcrumb configuration
const breadcrumbConfig = writable<{
	showHome?: boolean;
	homeIcon?: boolean;
	items?: BreadcrumbItem[];
}>({});

// Derived store that automatically generates breadcrumbs from URL
export const breadcrumbItems = derived(
	[page, customBreadcrumbs, breadcrumbConfig],
	([$page, $customBreadcrumbs, $config]) => {
		// If custom breadcrumbs are set, use them
		if ($customBreadcrumbs.length > 0) {
			return $customBreadcrumbs;
		}

		// Generate breadcrumbs based on current page including query parameters
		return generateBreadcrumbsFromPath($page.url.pathname, $page.url.search);
	}
);

// Derived store for home item based on current path
export const homeItem = derived(
	page,
	($page) => {
		// Always use default home
		return {
			label: 'Home',
			href: '/',
			icon: 'home'
		};
	}
);

// Derived store for showHome setting based on current path
export const showHome = derived(
	page,
	($page) => {
		// Always show home
		return true;
	}
);

// Derived store for breadcrumb configuration
export const breadcrumbSettings = derived(
	breadcrumbConfig,
	($config) => ({
		showHome: $config.showHome ?? true,
		homeIcon: $config.homeIcon ?? true,
		...$config
	})
);

/**
 * Generate breadcrumbs from URL path and query parameters
 */
function generateBreadcrumbsFromPath(pathname: string, search: string): BreadcrumbItem[] {
	const segments = pathname.split('/').filter(Boolean);
	const breadcrumbs: BreadcrumbItem[] = [];

	// Build breadcrumbs from path segments
	let currentPath = '';
	
	for (let i = 0; i < segments.length; i++) {
		const segment = segments[i];
		currentPath += `/${segment}`;
		
		// Convert segment to uppercase for display
		const label = segment.toUpperCase();
		
		// Use a generic icon for all segments
		const icon = 'file';
		
		breadcrumbs.push({
			label,
			href: currentPath,
			icon
		});
	}

	// Add query parameters as a separate breadcrumb if they exist
	if (search) {
		const queryParams = new URLSearchParams(search);
		const paramEntries = Array.from(queryParams.entries());
		
		if (paramEntries.length > 0) {
			// Create a label showing the query parameters
			const paramLabels = paramEntries.map(([key, value]) => `${key}=${value}`).join('&');
			const queryLabel = `?${paramLabels}`;
			
			breadcrumbs.push({
				label: queryLabel,
				href: `${pathname}${search}`,
				icon: 'search'
			});
		}
	}

	return breadcrumbs;
}


/**
 * Set custom breadcrumbs for current page
 */
export function setBreadcrumbs(items: BreadcrumbItem[]) {
	log.debug('Setting custom breadcrumbs', { items });
	customBreadcrumbs.set(items);
}

/**
 * Clear custom breadcrumbs (will fall back to auto-generated ones)
 */
export function clearBreadcrumbs() {
	log.debug('Clearing custom breadcrumbs');
	customBreadcrumbs.set([]);
}

/**
 * Set breadcrumb configuration
 */
export function setBreadcrumbConfig(config: {
	showHome?: boolean;
	homeIcon?: boolean;
	items?: BreadcrumbItem[];
}) {
	log.debug('Setting breadcrumb config', { config });
	breadcrumbConfig.set(config);
}

/**
 * Reset breadcrumb configuration to defaults
 */
export function resetBreadcrumbConfig() {
	log.debug('Resetting breadcrumb config');
	breadcrumbConfig.set({});
}

// Actions for common breadcrumb patterns
// Most breadcrumbs are now auto-generated from URL paths
export const breadcrumbActions = {
	/**
	 * Override breadcrumbs for specific cases where auto-generation isn't sufficient
	 * For example, when you need to show a specific task title instead of "Task Details"
	 */
	setCustomBreadcrumbs(items: BreadcrumbItem[]) {
		setBreadcrumbs(items);
	},

	/**
	 * Clear custom breadcrumbs to fall back to auto-generated ones
	 */
	clearCustomBreadcrumbs() {
		clearBreadcrumbs();
	},

	/**
	 * Set breadcrumbs for a specific task with its title
	 */
	setTaskBreadcrumbs(taskTitle: string, taskId: string) {
		setBreadcrumbs([
			{ label: 'Tasks', href: '/tasks', icon: 'clipboard' },
			{ label: taskTitle, href: `/tasks/${taskId}`, icon: 'clipboard-text' }
		]);
	},

	/**
	 * Set breadcrumbs for task edit with title
	 */
	setTaskEditBreadcrumbs(taskTitle: string, taskId: string) {
		setBreadcrumbs([
			{ label: 'Tasks', href: '/tasks', icon: 'clipboard' },
			{ label: taskTitle, href: `/tasks/${taskId}`, icon: 'clipboard-text' },
			{ label: 'Edit', icon: 'edit' }
		]);
	},

	/**
	 * Set breadcrumbs for projects page
	 */
	setProjectBreadcrumbs() {
		setBreadcrumbs([
			{ label: 'Projects', href: '/projects', icon: 'folder' }
		]);
	},

	/**
	 * Set breadcrumbs for project details
	 */
	setProjectDetailsBreadcrumbs(projectName: string) {
		setBreadcrumbs([
			{ label: 'Projects', href: '/projects', icon: 'folder' },
			{ label: projectName, icon: 'folder-open' }
		]);
	},

	/**
	 * Set breadcrumbs for create project
	 */
	setCreateProjectBreadcrumbs() {
		setBreadcrumbs([
			{ label: 'Projects', href: '/projects', icon: 'folder' },
			{ label: 'Create Project', icon: 'plus' }
		]);
	},

	/**
	 * Set breadcrumbs for project edit
	 */
	setProjectEditBreadcrumbs(projectName: string) {
		setBreadcrumbs([
			{ label: 'Projects', href: '/projects', icon: 'folder' },
			{ label: projectName, href: `/projects/${projectName}`, icon: 'folder-open' },
			{ label: 'Edit', icon: 'edit' }
		]);
	},

	/**
	 * Set breadcrumbs for terminal page
	 */
	setTerminalBreadcrumbs() {
		setBreadcrumbs([
			{ label: 'Terminal', href: '/terminal', icon: 'terminal' }
		]);
	}
};
