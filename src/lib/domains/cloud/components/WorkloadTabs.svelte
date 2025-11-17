<!-- WorkloadTabs - Provider-agnostic tab navigation for workloads -->
<script lang="ts">
  import { page } from '$app/stores';
  import { goto } from '$app/navigation';
  import { Button } from '@/lib/components/ui/button';

  // Tab navigation - provider-agnostic
  const tabs = [
    { id: 'overview', label: 'Overview', route: '/cloud/workloads', icon: '☸️' },
    { id: 'pods', label: 'Pods', route: '/cloud/workloads/pods', icon: '📦' },
    { id: 'services', label: 'Services', route: '/cloud/workloads/services', icon: '🔗' },
    { id: 'deployments', label: 'Deployments', route: '/cloud/workloads/deployments', icon: '🚀' },
    { id: 'statefulsets', label: 'StatefulSets', route: '/cloud/workloads/statefulsets', icon: '🗄️' },
    { id: 'daemonsets', label: 'DaemonSets', route: '/cloud/workloads/daemonsets', icon: '👹' },
    { id: 'jobs', label: 'Jobs', route: '/cloud/workloads/jobs', icon: '⚙️' },
    { id: 'cronjobs', label: 'CronJobs', route: '/cloud/workloads/cronjobs', icon: '⏰' },
    { id: 'configmaps', label: 'ConfigMaps', route: '/cloud/configmaps', icon: '⚙️' },
    { id: 'secrets', label: 'Secrets', route: '/cloud/secrets', icon: '🔐' },
    { id: 'ingress', label: 'Ingress', route: '/cloud/ingress', icon: '🌐' }
  ];

  function navigateToTab(tabRoute: string) {
    goto(tabRoute);
  }

  function isActiveTab(tabRoute: string): boolean {
    const currentPath = $page.url.pathname;
    // Special handling for overview tab (exact match for /cloud/workloads)
    if (tabRoute === '/cloud/workloads') {
      return currentPath === '/cloud/workloads';
    }
    // For configmaps, secrets, ingress, and cronjobs, check exact match or detail pages
    if (tabRoute === '/cloud/configmaps') {
      return currentPath === '/cloud/configmaps' || currentPath.startsWith('/cloud/configmaps/');
    }
    if (tabRoute === '/cloud/secrets') {
      return currentPath === '/cloud/secrets' || currentPath.startsWith('/cloud/secrets/');
    }
    if (tabRoute === '/cloud/ingress') {
      return currentPath === '/cloud/ingress' || currentPath.startsWith('/cloud/ingress/');
    }
    // For other tabs, check if current path starts with the tab route
    return currentPath.startsWith(tabRoute);
  }
</script>

<!-- Tab Navigation -->
<div class="bg-background border-b">
  <div class="px-6">
    <nav class="flex space-x-8">
      {#each tabs as tab}
        <Button
          variant="ghost"
          onclick={() => navigateToTab(tab.route)}
          class="flex items-center space-x-2 py-4 px-1 border-b-2 font-medium text-sm transition-colors rounded-none h-auto {isActiveTab(tab.route) 
            ? 'border-primary text-primary' 
            : 'border-transparent text-muted-foreground hover:text-foreground hover:border-muted-foreground'}"
        >
          <span class="text-lg">{tab.icon}</span>
          <span>{tab.label}</span>
        </Button>
      {/each}
    </nav>
  </div>
</div>

