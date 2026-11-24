<script lang="ts">
  import { useQueryClient } from '@tanstack/svelte-query';
  import { Card, CardContent } from '$lib/components/ui/card';
  import { Switch } from '$lib/components/ui/switch';
  import { Label } from '$lib/components/ui/label';
  import { Button } from '$lib/components/ui/button';
  import { createLanEnabledQuery } from '../queries/deviceQueries';
  import { setLanEnabled } from '../api/deviceApi';
  import { queryKeys } from '$lib/domains/shared/query/keys';
  import { relaunch } from '@tauri-apps/plugin-process';
  import { toast } from '$lib/utils/toast';

  interface Props {
    /** One-line form for simple mode, where it only shows up when LAN is off. */
    compact?: boolean;
  }

  let { compact = false }: Props = $props();

  const queryClient = useQueryClient();
  const lanQ = createLanEnabledQuery();
  let saving = $state(false);
  let changed = $state(false);

  async function onToggle(checked: boolean) {
    saving = true;
    try {
      await setLanEnabled(checked);
      changed = true;
      await queryClient.invalidateQueries({ queryKey: queryKeys.devices.lanEnabled });
    } catch (err) {
      toast.error('Failed to update LAN setting', err instanceof Error ? err.message : String(err));
    } finally {
      saving = false;
    }
  }
</script>

{#if compact}
  {#if lanQ.data === false}
    <Card class="border-destructive/40">
      <CardContent class="flex flex-wrap items-center justify-between gap-3 p-3">
        <p class="text-sm">
          Pairing over the local network is off — a phone can't reach Portal Desktop.
        </p>
        <Button size="sm" disabled={saving} onclick={() => onToggle(true)}>Turn on</Button>
      </CardContent>
    </Card>
  {/if}
{:else}
  <Card>
    <CardContent class="flex items-center justify-between gap-4 pt-6">
      <div>
        <Label for="lan-toggle">Allow pairing over the local network</Label>
        <p class="text-xs text-muted-foreground">
          Enables UDP discovery and the LAN listener a phone connects to. Takes effect after
          restarting Portal Desktop.
        </p>
      </div>
      {#if changed}
        <Button size="sm" variant="outline" onclick={() => relaunch()}>Restart now</Button>
      {/if}
      <Switch
        id="lan-toggle"
        checked={lanQ.data ?? false}
        disabled={lanQ.isPending || saving}
        onCheckedChange={onToggle}
      />
    </CardContent>
  </Card>
{/if}
