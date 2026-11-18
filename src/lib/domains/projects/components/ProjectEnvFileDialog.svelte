<!-- Link a .env file that lives in a sub-folder (path relative to the project root). -->
<script lang="ts">
  import {
    Dialog,
    DialogContent,
    DialogDescription,
    DialogFooter,
    DialogHeader,
    DialogTitle,
  } from '$lib/components/ui/dialog';
  import { Input } from '$lib/components/ui/input';
  import { Button } from '$lib/components/ui/button';
  import { open } from '@tauri-apps/plugin-dialog';

  interface Props {
    projectPath?: string;
    onSubmit: (path: string) => Promise<void>;
    onClose: () => void;
  }

  const { projectPath, onSubmit, onClose }: Props = $props();

  let path = $state('');
  let busy = $state(false);

  async function browse() {
    const picked = await open({ multiple: false, directory: false, defaultPath: projectPath });
    if (typeof picked === 'string') path = picked;
  }

  async function submit() {
    if (!path.trim()) return;
    busy = true;
    try {
      await onSubmit(path.trim());
    } finally {
      busy = false;
    }
  }
</script>

<Dialog open onOpenChange={(o) => !o && onClose()}>
  <DialogContent class="sm:max-w-md">
    <DialogHeader>
      <DialogTitle>Link a .env file</DialogTitle>
      <DialogDescription>
        Path relative to the project root, e.g. <span class="font-mono">apps/web/.env.local</span>.
      </DialogDescription>
    </DialogHeader>
    <div class="flex gap-2">
      <Input
        bind:value={path}
        placeholder="apps/web/.env"
        class="font-mono"
        onkeydown={(e) => e.key === 'Enter' && submit()}
      />
      <Button variant="outline" onclick={browse}>Browse…</Button>
    </div>
    <DialogFooter>
      <Button variant="ghost" onclick={onClose}>Cancel</Button>
      <Button onclick={submit} disabled={busy || !path.trim()}>Link file</Button>
    </DialogFooter>
  </DialogContent>
</Dialog>
