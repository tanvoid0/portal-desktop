<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import { Button } from '$lib/components/ui/button';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { formatAbsolute, formatBytes, formatRelative } from '../utils/syncVerdict';

  interface Props {
    open: boolean;
    local: { modifiedAt: string | null; size: number; baseVersion: number };
    cloud: { version: number; updatedAt: string; size: number; deviceId: string };
    busy?: boolean;
    onKeepLocal: () => void;
    onTakeCloud: () => void;
  }

  let {
    open = $bindable(),
    local,
    cloud,
    busy = false,
    onKeepLocal,
    onTakeCloud,
  }: Props = $props();
</script>

<Dialog bind:open>
  <DialogContent class="max-w-[min(100vw-2rem,40rem)]">
    <DialogHeader>
      <DialogTitle>Cloud has changes from another session</DialogTitle>
      <DialogDescription>
        This machine was last synced at version {local.baseVersion}, but the cloud is now at version
        {cloud.version}. Choose which copy to keep.
      </DialogDescription>
    </DialogHeader>
    <div class="grid grid-cols-1 gap-3 sm:grid-cols-2">
      <Card>
        <CardHeader class="pb-2"><CardTitle class="text-base">This machine</CardTitle></CardHeader>
        <CardContent class="space-y-1 text-sm">
          <div>{formatAbsolute(local.modifiedAt)}</div>
          <div class="text-muted-foreground">{formatRelative(local.modifiedAt)}</div>
          <div>{formatBytes(local.size)}</div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader class="pb-2"><CardTitle class="text-base">Cloud</CardTitle></CardHeader>
        <CardContent class="space-y-1 text-sm">
          <div>{formatAbsolute(cloud.updatedAt)}</div>
          <div class="text-muted-foreground">{formatRelative(cloud.updatedAt)}</div>
          <div>{formatBytes(cloud.size)} - v{cloud.version}</div>
          <div class="truncate text-muted-foreground" title={cloud.deviceId}>
            from {cloud.deviceId}
          </div>
        </CardContent>
      </Card>
    </div>
    <DialogFooter class="gap-2 sm:flex-wrap">
      <Button variant="outline" disabled={busy} onclick={() => (open = false)}>Cancel</Button>
      <Button variant="destructive" disabled={busy} onclick={onKeepLocal}>
        Keep this machine (overwrite cloud)
      </Button>
      <Button disabled={busy} onclick={onTakeCloud}>Take cloud (replace this machine)</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
