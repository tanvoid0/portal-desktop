<!-- PodsTable - Pod-specific table using BaseResourceTable -->
<script lang="ts">
  import BaseResourceTable from '../../core/components/BaseResourceTable.svelte';
  import type { ICloudResource } from '../../core/types';
  import { ResourceType } from '../../core/types';
  import { goto } from '$app/navigation';
  
  interface Props {
    pods: ICloudResource[];
    onViewPod?: (pod: ICloudResource) => void;
    onViewLogs?: (pod: ICloudResource) => void;
    onDeletePod?: (pod: ICloudResource) => void;
    emptyMessage?: string;
  }
  
  let {
    pods = [],
    onViewPod,
    onViewLogs,
    onDeletePod,
    emptyMessage = 'No pods found'
  }: Props = $props();
  
  // Pod-specific columns
  const podColumns = [
    { key: 'name', label: 'Name', width: 'w-1/4' },
    { key: 'status', label: 'Status', width: 'w-1/8' },
    { key: 'ready', label: 'Ready', width: 'w-1/8' },
    { key: 'restarts', label: 'Restarts', width: 'w-1/8' },
    { key: 'age', label: 'Age', width: 'w-1/8' },
    { key: 'namespace', label: 'Namespace', width: 'w-1/6' },
    { key: 'actions', label: 'Actions', width: 'w-1/6' }
  ];
  
  function handlePodClick(pod: ICloudResource) {
    if (onViewPod) {
      onViewPod(pod);
    } else {
      goto(`/cloud/workloads/pods/${pod.name}?namespace=${pod.namespace}`);
    }
  }
  
  function handleViewLogs(pod: ICloudResource) {
    if (onViewLogs) {
      onViewLogs(pod);
    } else {
      goto(`/cloud/workloads/pods/${pod.name}?namespace=${pod.namespace}&tab=logs`);
    }
  }
</script>

<BaseResourceTable
  resources={pods}
  resourceType={ResourceType.POD}
  columns={podColumns}
  onResourceClick={handlePodClick}
  onViewLogs={handleViewLogs}
  onDelete={onDeletePod}
  {emptyMessage}
/>

