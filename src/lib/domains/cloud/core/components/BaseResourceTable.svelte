<!-- BaseResourceTable - Generic table component that works with any provider's resources -->
<script lang="ts">
  import type { ICloudResource, ResourceType } from '../../types';
  import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/lib/components/ui/table';
  import { Button } from '@/lib/components/ui/button';
  
  interface Column {
    key: string;
    label: string;
    width?: string;
  }
  
  interface Props {
    resources: ICloudResource[];
    resourceType: ResourceType;
    columns?: Column[];
    onResourceClick?: (resource: ICloudResource) => void;
    onViewLogs?: (resource: ICloudResource) => void;
    onDelete?: (resource: ICloudResource) => void;
    emptyMessage?: string;
  }
  
  let {
    resources = [],
    resourceType,
    columns = [],
    onResourceClick,
    onViewLogs,
    onDelete,
    emptyMessage = 'No resources found'
  }: Props = $props();
  
  // Default columns if none provided
  const defaultColumns = $derived.by(() => {
    if (columns.length > 0) return columns;
    
    // Default columns based on resource type
    return [
      { key: 'name', label: 'Name', width: 'w-1/3' },
      { key: 'status', label: 'Status', width: 'w-1/8' },
      { key: 'namespace', label: 'Namespace', width: 'w-1/6' },
      { key: 'age', label: 'Age', width: 'w-1/6' },
      { key: 'actions', label: 'Actions', width: 'w-1/6' }
    ];
  });
  
  function getStatusColor(status: string): string {
    const statusLower = status.toLowerCase();
    switch (statusLower) {
      case 'running':
        return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
      case 'succeeded':
        return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200';
      case 'pending':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200';
      case 'failed':
      case 'error':
        return 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200';
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900 dark:text-gray-200';
    }
  }
  
  function getCellValue(resource: ICloudResource, key: string): string {
    if (key === 'name') return resource.name;
    if (key === 'status') return resource.status;
    if (key === 'namespace') return resource.namespace;
    if (key === 'age') return resource.metadata.age || 'N/A';
    if (key === 'ready') return resource.metadata.ready || 'N/A';
    if (key === 'restarts') return String(resource.metadata.restarts || 0);
    if (key === 'type') return resource.metadata.type || 'N/A';
    if (key === 'clusterIP') return resource.metadata.clusterIP || 'N/A';
    if (key === 'ports') {
      const ports = resource.metadata.ports;
      if (Array.isArray(ports) && ports.length > 0) {
        return ports.map((p: any) => `${p.port}${p.targetPort ? `:${p.targetPort}` : ''}`).join(', ');
      }
      return resource.metadata.ports || 'N/A';
    }
    if (key === 'replicas') {
      const replicas = resource.metadata.replicas || resource.metadata.desired || 0;
      const ready = resource.metadata.readyReplicas || resource.metadata.ready || 0;
      return replicas ? `${ready}/${replicas}` : 'N/A';
    }
    if (key === 'desired') {
      return String(resource.metadata.desired || 0);
    }
    if (key === 'current') {
      return String(resource.metadata.current || 0);
    }
    if (key === 'ready') {
      return String(resource.metadata.ready || 0);
    }
    if (key === 'dataCount') {
      return String(resource.metadata.dataCount || 0);
    }
    if (key === 'completions') {
      return String(resource.metadata.completions || 0);
    }
    if (key === 'succeeded') {
      return String(resource.metadata.succeeded || 0);
    }
    if (key === 'failed') {
      return String(resource.metadata.failed || 0);
    }
    if (key === 'schedule') {
      return resource.metadata.schedule || 'N/A';
    }
    if (key === 'suspend') {
      return resource.metadata.suspend ? 'Yes' : 'No';
    }
    if (key === 'active') {
      return String(resource.metadata.active || 0);
    }
    if (key === 'lastSchedule') {
      return resource.metadata.last_schedule_time || 'Never';
    }
    if (key === 'class') {
      return resource.metadata.class || 'N/A';
    }
    if (key === 'addresses') {
      const addrs = resource.metadata.addresses;
      if (Array.isArray(addrs) && addrs.length > 0) {
        return addrs.join(', ');
      }
      return 'N/A';
    }
    if (key === 'ports') {
      const ports = resource.metadata.ports;
      if (Array.isArray(ports) && ports.length > 0) {
        return ports.join(', ');
      }
      return 'N/A';
    }
    return resource.metadata[key] || '';
  }
</script>

<Table>
  <TableHeader>
    <TableRow>
      {#each defaultColumns as column}
        <TableHead class={column.width}>{column.label}</TableHead>
      {/each}
    </TableRow>
  </TableHeader>
  <TableBody>
    {#if resources.length === 0}
      <TableRow>
        <TableCell colspan={defaultColumns.length} class="text-center py-8 text-muted-foreground">
          {emptyMessage}
        </TableCell>
      </TableRow>
    {:else}
      {#each resources as resource, index (resource.id)}
        <TableRow 
          class="hover:bg-muted/50 cursor-pointer" 
          onclick={() => onResourceClick?.(resource)}
          data-index={index}
        >
          {#each defaultColumns as column}
            {@const value = getCellValue(resource, column.key)}
            <TableCell>
              {#if column.key === 'name'}
                <div class="flex items-center space-x-2">
                  <div class="w-2 h-2 rounded-full {
                    resource.status === 'running' || resource.status === 'succeeded' 
                      ? 'bg-green-500' 
                      : resource.status === 'pending' 
                      ? 'bg-yellow-500' 
                      : 'bg-red-500'
                  }"></div>
                  <span class="font-medium">{value}</span>
                </div>
              {:else if column.key === 'status'}
                <span class="inline-flex items-center px-2.5 py-1 rounded-full text-xs font-medium {getStatusColor(value)}">
                  {value}
                </span>
              {:else if column.key === 'actions'}
                <div class="flex items-center space-x-1">
                  {#if onViewLogs && resource.getLogs}
                    <Button
                      variant="ghost"
                      size="sm"
                      onclick={(e) => { e.stopPropagation(); onViewLogs(resource); }}
                      title="View Logs"
                    >
                      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"></path>
                      </svg>
                    </Button>
                  {/if}
                  {#if onDelete && resource.delete}
                    <Button
                      variant="ghost"
                      size="sm"
                      onclick={(e) => { e.stopPropagation(); onDelete(resource); }}
                      title="Delete"
                    >
                      <svg class="w-4 h-4 text-destructive" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"></path>
                      </svg>
                    </Button>
                  {/if}
                </div>
              {:else}
                <span class="text-sm">{value}</span>
              {/if}
            </TableCell>
          {/each}
        </TableRow>
      {/each}
    {/if}
  </TableBody>
</Table>

