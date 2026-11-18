<!-- Connection fields for database credentials (stored as plaintext metadata; the secret stays the password). -->
<script lang="ts">
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import Select from '$lib/components/ui/select.svelte';
  import type { CredentialMetadata } from '../types';

  interface Props {
    metadata: Partial<CredentialMetadata>;
    disabled?: boolean;
  }

  let { metadata = $bindable(), disabled: loading = false }: Props = $props();

  // Database Explorer discovery reads these from plaintext metadata (host is required).
  const dbEngines = [
    { value: 'postgres', label: 'PostgreSQL' },
    { value: 'mysql', label: 'MySQL / MariaDB' },
    { value: 'mongodb', label: 'MongoDB' },
    { value: 'redis', label: 'Redis' },
  ];
</script>

<div class="grid grid-cols-1 gap-4 sm:grid-cols-2">
  <div class="space-y-2">
    <Label>Engine</Label>
    <Select
      options={dbEngines}
      defaultValue={metadata.provider ?? ''}
      placeholder="Select engine"
      onSelect={(v) => (metadata.provider = v)}
      disabled={loading}
    />
  </div>
  <div class="space-y-2">
    <Label for="db-host">Host</Label>
    <Input id="db-host" placeholder="localhost" bind:value={metadata.host} disabled={loading} />
  </div>
  <div class="space-y-2">
    <Label for="db-port">Port</Label>
    <Input
      id="db-port"
      type="number"
      placeholder="default"
      bind:value={metadata.port}
      disabled={loading}
    />
  </div>
  <div class="space-y-2">
    <Label for="db-name">Database</Label>
    <Input id="db-name" bind:value={metadata.database} disabled={loading} />
  </div>
  <div class="space-y-2 sm:col-span-2">
    <Label for="db-user">Username</Label>
    <Input id="db-user" bind:value={metadata.username} disabled={loading} />
  </div>
</div>
