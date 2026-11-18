<!-- Environment variable editor with optional seeding from a project .env* file -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import Select from '$lib/components/ui/select.svelte';
  import type { EnvFile } from '../types';

  interface Props {
    envVars: Record<string, string>;
    envFiles: EnvFile[];
    selectedEnvFile: string;
    onSelectFile: (fileName: string) => void;
  }
  let { envVars = $bindable(), envFiles, selectedEnvFile, onSelectFile }: Props = $props();

  function addEnvVar() {
    envVars = { ...envVars, '': '' };
  }

  function removeEnvVar(key: string) {
    const newEnvVars = { ...envVars };
    delete newEnvVars[key];
    envVars = newEnvVars;
  }

  function updateEnvVar(oldKey: string, newKey: string, value: string) {
    const newEnvVars = { ...envVars };
    if (oldKey !== newKey) {
      delete newEnvVars[oldKey];
    }
    newEnvVars[newKey] = value;
    envVars = newEnvVars;
  }
</script>

<div class="divider-edge-t divider-edge-full pt-6">
  <div class="mb-4 flex items-center justify-between">
    <div>
      <h3 class="text-lg font-semibold">Environment Variables</h3>
      <p class="text-sm text-muted-foreground">
        Optional environment variables for your deployment
      </p>
    </div>
    <Button type="button" variant="outline" size="sm" onclick={addEnvVar}>Add Variable</Button>
  </div>
  {#if envFiles.length > 0}
    <div class="mb-3">
      <Label for="envFile">Load from .env file</Label>
      <Select
        options={[
          { value: '', label: 'None (manual)' },
          ...envFiles.map((f) => ({
            value: f.name,
            label: `${f.name} — ${f.vars.length} var${f.vars.length === 1 ? '' : 's'}`,
          })),
        ]}
        defaultValue={selectedEnvFile}
        onSelect={onSelectFile}
      />
      <p class="mt-1 text-xs text-muted-foreground">
        Fills the list below from the chosen file. Injected at run time, not baked into the image.
      </p>
    </div>
  {/if}
  <div class="space-y-2">
    {#each Object.entries(envVars) as [key, value]}
      <div class="flex gap-2">
        <Input
          placeholder="Variable name"
          value={key}
          oninput={(e) => updateEnvVar(key, e.currentTarget.value, value)}
          class="flex-1"
        />
        <Input
          placeholder="Value"
          {value}
          oninput={(e) => updateEnvVar(key, key, e.currentTarget.value)}
          class="flex-1"
        />
        <Button type="button" variant="outline" size="sm" onclick={() => removeEnvVar(key)}>
          Remove
        </Button>
      </div>
    {:else}
      <p class="py-4 text-center text-sm text-muted-foreground">No environment variables added</p>
    {/each}
  </div>
</div>
