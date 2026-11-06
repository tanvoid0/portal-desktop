/**
 * Breadcrumb store for managing navigation state
 * Provides centralized breadcrumb management across the application
 */

import { writable, derived, readable, toStore } from 'svelte/store';
import { page } from '$app/state';
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
// Path the custom items were set on; they are ignored once the user navigates elsewhere
let customPath = '';
// `page` is rune state; this makes the derived store re-run on navigation
const currentHref = toStore(() => page.url.href);

// Store for page-specific breadcrumb configuration
const breadcrumbConfig = writable<{
  showHome?: boolean;
  homeIcon?: boolean;
  items?: BreadcrumbItem[];
}>({});

// Derived store that automatically generates breadcrumbs from URL
export const breadcrumbItems = derived(
  [customBreadcrumbs, breadcrumbConfig, currentHref],
  ([$customBreadcrumbs]) => {
    // Use custom breadcrumbs only on the page that set them
    if ($customBreadcrumbs.length > 0 && customPath === page.url.pathname) {
      return $customBreadcrumbs;
    }

    // Generate breadcrumbs based on current page including query parameters
    return generateBreadcrumbsFromPath(page.url.pathname, page.url.search, page.route.id);
  }
);

// Home item is always static
export const homeItem = readable<BreadcrumbItem>({
  label: 'Home',
  href: '/',
  icon: 'home',
});

// Always show home
export const showHome = readable(true);

// Derived store for breadcrumb configuration
export const breadcrumbSettings = derived(breadcrumbConfig, ($config) => ({
  showHome: $config.showHome ?? true,
  homeIcon: $config.homeIcon ?? true,
  ...$config,
}));

// Acronyms/brands the plain humanizer would get wrong
const SEGMENT_LABELS: Record<string, string> = {
  ides: 'IDEs',
  'framework-ides': 'Framework IDEs',
  sdk: 'SDK',
  github: 'GitHub',
  ai: 'AI',
  'pipeline-runs': 'Action Runs',
  'frameworks-languages': 'Frameworks & Languages',
};

/**
 * Generate breadcrumbs from URL path and query parameters
 */
function generateBreadcrumbsFromPath(
  pathname: string,
  search: string,
  routeId?: string | null
): BreadcrumbItem[] {
  const segments = pathname.split('/').filter(Boolean);
  const routeSegments = (routeId ?? '').split('/').filter(Boolean);
  const breadcrumbs: BreadcrumbItem[] = [];

  // Build breadcrumbs from path segments
  let currentPath = '';

  for (let i = 0; i < segments.length; i++) {
    const segment = segments[i];
    currentPath += `/${segment}`;

    // Static segments are humanized; [param] values are shown as-is ("#n" if numeric)
    const isParam = routeSegments.length === segments.length && /^\[.*\]$/.test(routeSegments[i]);
    const decoded = decodeURIComponent(segment);
    const label = isParam
      ? /^\d+$/.test(decoded)
        ? `#${decoded}`
        : decoded
      : (SEGMENT_LABELS[decoded] ??
        decoded.replace(/[-_]+/g, ' ').replace(/^\w/, (c) => c.toUpperCase()));

    // Use a generic icon for all segments
    const icon = 'file';

    breadcrumbs.push({
      label,
      href: currentPath,
      icon,
    });
  }

  // Add query parameters as a separate breadcrumb if they exist
  if (search) {
    const queryParams = new URLSearchParams(search);
    // `tab` is page-local view state (tabs show their own label), not a location
    const paramEntries = Array.from(queryParams.entries()).filter(
      ([key]) => !['tab', 'from', 'q', 'status'].includes(key)
    );

    if (paramEntries.length > 0) {
      // Create a label showing the query parameters
      const paramLabels = paramEntries.map(([key, value]) => `${key}=${value}`).join('&');
      const queryLabel = `?${paramLabels}`;

      breadcrumbs.push({
        label: queryLabel,
        href: `${pathname}${search}`,
        icon: 'search',
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
  customPath = page.url.pathname;
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
      { label: taskTitle, href: `/tasks/${taskId}`, icon: 'clipboard-text' },
    ]);
  },

  /**
   * Set breadcrumbs for task edit with title
   */
  setTaskEditBreadcrumbs(taskTitle: string, taskId: string) {
    setBreadcrumbs([
      { label: 'Tasks', href: '/tasks', icon: 'clipboard' },
      { label: taskTitle, href: `/tasks/${taskId}`, icon: 'clipboard-text' },
      { label: 'Edit', icon: 'edit' },
    ]);
  },

  /**
   * Set breadcrumbs for projects page
   */
  setProjectBreadcrumbs() {
    setBreadcrumbs([{ label: 'Projects', href: '/projects', icon: 'folder' }]);
  },

  /**
   * Set breadcrumbs for project details
   */
  setProjectDetailsBreadcrumbs(projectName: string) {
    setBreadcrumbs([
      { label: 'Projects', href: '/projects', icon: 'folder' },
      { label: projectName, icon: 'folder-open' },
    ]);
  },

  /**
   * Set breadcrumbs for create project
   */
  setCreateProjectBreadcrumbs() {
    setBreadcrumbs([
      { label: 'Projects', href: '/projects', icon: 'folder' },
      { label: 'Create Project', icon: 'plus' },
    ]);
  },

  /**
   * Set breadcrumbs for project edit
   */
  setProjectEditBreadcrumbs(projectName: string, projectId: string | number) {
    setBreadcrumbs([
      { label: 'Projects', href: '/projects', icon: 'folder' },
      { label: projectName, href: `/projects/${projectId}`, icon: 'folder-open' },
      { label: 'Edit', icon: 'edit' },
    ]);
  },

  /**
   * Set breadcrumbs for a project sub-page (e.g. "Create Pipeline", "Pipeline Run")
   */
  setProjectSubpageBreadcrumbs(projectName: string, projectId: string | number, label: string) {
    setBreadcrumbs([
      { label: 'Projects', href: '/projects', icon: 'folder' },
      { label: projectName, href: `/projects/${projectId}`, icon: 'folder-open' },
      { label, icon: 'file' },
    ]);
  },

  /**
   * Set breadcrumbs for terminal page
   */
  setTerminalBreadcrumbs() {
    setBreadcrumbs([{ label: 'Terminal', href: '/terminal', icon: 'terminal' }]);
  },
};
