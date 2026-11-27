<!-- Deployment overview: info, replica progress bars, labels -->
<script lang="ts">
  import { Progress } from '$lib/components/ui/progress';
  import type { ICloudResource } from '$lib/domains/cloud/core/types';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import InfoCard from '../InfoCard.svelte';
  import LabelsCard from '../LabelsCard.svelte';

  interface Props {
    deployment: ICloudResource;
  }
  let { deployment }: Props = $props();

  const labels = $derived(deployment.metadata?.labels || {});
  const desired = $derived(deployment.metadata?.desired);
  const bars = $derived([
    { label: 'Ready', value: deployment.metadata?.available, color: 'bg-green-600' },
    { label: 'Up-to-date', value: deployment.metadata?.up_to_date, color: 'bg-blue-600' },
  ]);
</script>

<div class="grid grid-cols-1 gap-4 md:grid-cols-2">
  <InfoCard
    title="Deployment Information"
    status={{ text: deployment.status }}
    fields={[
      { label: 'Desired Replicas', value: desired || 0 },
      { label: 'Current Replicas', value: deployment.metadata?.current || 0 },
      {
        label: 'Available Replicas',
        value: `${deployment.metadata?.available || 0}/${desired || 0}`,
      },
      { label: 'Up-to-date Replicas', value: deployment.metadata?.up_to_date || 0 },
      { label: 'Age', value: deployment.metadata?.age || 'N/A' },
    ]}
  />

  <Card>
    <CardHeader>
      <CardTitle>Replica Status</CardTitle>
    </CardHeader>
    <CardContent>
      <div class="space-y-4">
        {#each bars as bar (bar.label)}
          <div>
            <div class="mb-2 flex justify-between">
              <span class="text-sm">{bar.label}</span>
              <span class="font-medium">{bar.value || 0}/{desired || 0}</span>
            </div>
            <Progress
              value={desired ? (bar.value / desired) * 100 : 0}
              class="bg-muted"
              indicatorClass={bar.color}
            />
          </div>
        {/each}
      </div>
    </CardContent>
  </Card>
</div>

<LabelsCard items={labels} />
