<!-- Resolved command line plus the sudo / Windows-admin hint. -->
<script lang="ts">
  import SectionCard from '$lib/components/shared/SectionCard.svelte';

  interface Props {
    command: string;
    requiresSudo: boolean;
  }

  let { command, requiresSudo }: Props = $props();

  const isWindows = navigator.userAgent.includes('Windows');
</script>

<SectionCard title="Command Preview" titleClass="" contentClass="space-y-2">
  <pre class="overflow-x-auto rounded-md bg-muted p-4 font-mono text-sm">{command}</pre>
  {#if requiresSudo}
    {#if isWindows}
      <div
        class="rounded-md border border-amber-200 bg-amber-50 p-3 dark:border-amber-800 dark:bg-amber-950"
      >
        <p class="text-sm text-amber-800 dark:text-amber-200">
          <strong>Windows Admin Required:</strong> This command requires administrator privileges. Make
          sure the terminal is running as administrator, or the command may fail.
        </p>
      </div>
    {:else}
      <div
        class="rounded-md border border-blue-200 bg-blue-50 p-3 dark:border-blue-800 dark:bg-blue-950"
      >
        <p class="text-sm text-blue-800 dark:text-blue-200">
          <strong>Note:</strong> This command will prompt for your password to run with sudo privileges.
        </p>
      </div>
    {/if}
  {/if}
</SectionCard>
