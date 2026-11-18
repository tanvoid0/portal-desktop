<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import Select from '$lib/components/ui/select.svelte';
  import type { EnvironmentVariable } from './serviceConfigTypes';

  let { environmentVars = $bindable() }: { environmentVars: EnvironmentVariable[] } = $props();

  const SCOPES = [
    { value: 'project', label: 'Project' },
    { value: 'session', label: 'Session' },
    { value: 'global', label: 'Global' },
  ];

  function add() {
    environmentVars = [...environmentVars, { name: '', value: '', scope: 'project' }];
  }
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h3 class="font-medium">Environment Variables</h3>
    <Button variant="outline" onclick={add}>Add Variable</Button>
  </div>

  <div class="space-y-3">
    {#each environmentVars as envVar, index}
      <div class="flex items-center gap-3 rounded-lg border p-3">
        <div class="flex-1">
          <Label for="env-name-{index}" class="text-xs">Name</Label>
          <Input id="env-name-{index}" bind:value={envVar.name} placeholder="VARIABLE_NAME" />
        </div>
        <div class="flex-1">
          <Label for="env-value-{index}" class="text-xs">Value</Label>
          <Input id="env-value-{index}" bind:value={envVar.value} placeholder="variable_value" />
        </div>
        <div class="w-32">
          <Label for="env-scope-{index}" class="text-xs">Scope</Label>
          <Select
            options={SCOPES}
            defaultValue={envVar.scope}
            onSelect={(value: string) => (envVar.scope = value as EnvironmentVariable['scope'])}
          />
        </div>
        <Button
          variant="ghost"
          size="sm"
          onclick={() => (environmentVars = environmentVars.filter((_, i) => i !== index))}
        >
          Remove
        </Button>
      </div>
    {/each}
  </div>
</div>
