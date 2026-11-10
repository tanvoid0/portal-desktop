<script lang="ts">
  import { createQuery, useQueryClient } from '@tanstack/svelte-query';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import { createLanEnabledQuery } from '../queries/deviceQueries';
  import { setLanEnabled } from '../api/deviceApi';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { toast } from '$lib/utils/toast';

  const queryClient = useQueryClient();
  const lanQ = createQuery(createLanEnabledQuery);
  let saving = $state(false);

  async function onToggle(checked: boolean) {
    saving = true;
    try {
      await setLanEnabled(checked);
      await queryClient.invalidateQueries({ queryKey: queryKeys.devices.lanEnabled });
    } catch (err) {
      toast.error('Failed to update LAN setting', err instanceof Error ? err.message : String(err));
    } finally {
      saving = false;
    }
  }
</script>

<Card>
  <CardContent class="flex items-center justify-between gap-4 pt-6">
    <div>
      <Label for="lan-toggle">Allow pairing over the local network</Label>
      <p class="text-xs text-muted-foreground">
        Enables UDP discovery and the LAN listener a phone connects to. Takes effect after
        restarting Portal Desktop.
      </p>
    </div>
    <Switch
      id="lan-toggle"
      checked={lanQ.data ?? false}
      disabled={lanQ.isPending || saving}
      onCheckedChange={onToggle}
    />
  </CardContent>
</Card>
