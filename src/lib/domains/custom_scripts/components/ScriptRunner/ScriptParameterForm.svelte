<!-- Parameter inputs for a custom script (file/folder/boolean/password/number/text). -->
<script lang="ts">
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Switch } from '$lib/components/ui/switch';
  import FilePicker from '$lib/components/ui/file-picker.svelte';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';
  import type { ScriptParameter } from '$lib/domains/custom_scripts/services/customScriptService';

  interface Props {
    parameters: ScriptParameter[];
    values: Record<string, string>;
    booleans: Record<string, boolean>;
  }

  let { parameters, values = $bindable(), booleans = $bindable() }: Props = $props();

  function setValue(name: string, value: string) {
    values = { ...values, [name]: value };
  }
</script>

<SectionCard
  title="Parameters"
  description="Configure script parameters"
  titleClass=""
  contentClass="space-y-4"
>
  {#each parameters as param (param.name)}
    <div class="space-y-2">
      <Label>
        {param.label}
        {#if param.required}
          <span class="text-red-500">*</span>
        {/if}
      </Label>
      {#if param.description}
        <p class="text-sm text-muted-foreground">
          {param.description}
        </p>
      {/if}

      {#if param.parameter_type === 'file'}
        <FilePicker
          value={values[param.name] || ''}
          label=""
          description=""
          filters={param.file_filters ? [{ name: 'Files', extensions: param.file_filters }] : []}
          selectFolder={false}
          required={param.required}
          onChange={(path) => setValue(param.name, path)}
        />
      {:else if param.parameter_type === 'folder'}
        <FilePicker
          value={values[param.name] || ''}
          label=""
          description=""
          selectFolder={true}
          required={param.required}
          onChange={(path) => setValue(param.name, path)}
        />
      {:else if param.parameter_type === 'boolean'}
        <div class="flex items-center gap-2">
          <Switch
            checked={booleans[param.name] ?? false}
            onCheckedChange={(checked) => {
              booleans = { ...booleans, [param.name]: checked };
            }}
          />
          <span class="text-sm text-muted-foreground">
            {booleans[param.name] ? 'Enabled' : 'Disabled'}
          </span>
        </div>
      {:else}
        <Input
          type={param.parameter_type === 'password' || param.parameter_type === 'number'
            ? param.parameter_type
            : 'text'}
          value={values[param.name] || ''}
          oninput={(e) => setValue(param.name, (e.target as HTMLInputElement).value)}
          placeholder={param.default_value || `Enter ${param.label.toLowerCase()}`}
          required={param.required}
        />
      {/if}
    </div>
  {/each}
</SectionCard>
