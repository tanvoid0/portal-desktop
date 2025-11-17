<!-- PodsStatistics - Shows pod count breakdown by status -->
<script lang="ts">
  import type { ICloudResource } from '../../core/types';
  import { ResourceStatus } from '../../core/types';
  import { Card, CardContent, CardHeader, CardTitle } from '@/lib/components/ui/card';
  
  interface Props {
    pods: ICloudResource[];
  }
  
  let { pods = [] }: Props = $props();
  
  const stats = $derived({
    total: pods.length,
    running: pods.filter(p => p.status === ResourceStatus.RUNNING).length,
    pending: pods.filter(p => p.status === ResourceStatus.PENDING).length,
    failed: pods.filter(p => p.status === ResourceStatus.FAILED).length,
    succeeded: pods.filter(p => p.status === ResourceStatus.SUCCEEDED).length
  });
</script>

<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-5 gap-4">
  <Card>
    <CardHeader class="pb-2">
      <CardTitle class="text-sm font-medium text-muted-foreground">Total Pods</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="text-2xl font-bold">{stats.total}</div>
    </CardContent>
  </Card>
  
  <Card>
    <CardHeader class="pb-2">
      <CardTitle class="text-sm font-medium text-muted-foreground">Running</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="text-2xl font-bold text-green-600 dark:text-green-400">{stats.running}</div>
    </CardContent>
  </Card>
  
  <Card>
    <CardHeader class="pb-2">
      <CardTitle class="text-sm font-medium text-muted-foreground">Pending</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="text-2xl font-bold text-yellow-600 dark:text-yellow-400">{stats.pending}</div>
    </CardContent>
  </Card>
  
  <Card>
    <CardHeader class="pb-2">
      <CardTitle class="text-sm font-medium text-muted-foreground">Failed</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="text-2xl font-bold text-red-600 dark:text-red-400">{stats.failed}</div>
    </CardContent>
  </Card>
  
  <Card>
    <CardHeader class="pb-2">
      <CardTitle class="text-sm font-medium text-muted-foreground">Succeeded</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="text-2xl font-bold text-blue-600 dark:text-blue-400">{stats.succeeded}</div>
    </CardContent>
  </Card>
</div>

