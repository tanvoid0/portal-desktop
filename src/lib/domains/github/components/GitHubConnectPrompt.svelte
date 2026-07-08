<script lang="ts">
  import { goto } from "$app/navigation";
  import { PageEmpty } from "$lib/components/shell";
  import { githubService } from "$lib/domains/github";
  import type { GitHubConnectionStatus } from "$lib/domains/github";
  import { toast } from "$lib/utils/toast";
  import { FolderGit2 } from "@lucide/svelte";

  interface Props {
    status?: GitHubConnectionStatus | null;
    onConnected?: () => void | Promise<void>;
  }

  let { status = null, onConnected }: Props = $props();

  let connecting = $state(false);

  const description = $derived(
    status?.clientIdConfigured
      ? "Connect your GitHub account to browse repositories, clone projects, and manage issues."
      : "GitHub OAuth is not configured yet. Add your GitHub Client ID in Settings > GitHub, then connect your account.",
  );

  async function handleConnect() {
    if (!status?.clientIdConfigured) {
      goto("/settings/github");
      return;
    }

    try {
      connecting = true;
      await githubService.connectWithDeviceFlow();
      toast.success("GitHub connected");
      await onConnected?.();
    } catch (error) {
      toast.error(
        error instanceof Error ? error.message : "Failed to connect GitHub",
      );
    } finally {
      connecting = false;
    }
  }
</script>

<PageEmpty
  title="Connect GitHub"
  {description}
  icon={FolderGit2}
  actionLabel={status?.clientIdConfigured
    ? connecting
      ? "Connecting..."
      : "Connect GitHub"
    : "Open GitHub Settings"}
  onAction={handleConnect}
/>
