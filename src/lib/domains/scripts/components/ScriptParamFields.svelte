<!-- Parameter inputs for a script block: select / boolean / text per parameter. -->
<script lang="ts">
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Checkbox } from '$lib/components/ui/checkbox';
  import Select from '$lib/components/ui/select.svelte';
  import type { Block } from '$lib/domains/projects/pipelines';

  interface Props {
    parameters: Block['parameters'];
    values: Record<string, string>;
  }

  let { parameters, values = $bindable() }: Props = $props();
</script>

<div class="space-y-3">
  <Label class="text-sm font-medium">Parameters</Label>
  {#each parameters as param}
    <div class="grid grid-cols-3 items-start gap-4">
      <div class="flex items-center gap-2">
        <code class="font-mono text-sm">{param.name}</code>
        {#if param.required}
          <span class="text-red-500">*</span>
        {/if}
      </div>
      <div class="col-span-2">
        {#if param.type === 'select' && param.options}
          <Select
            options={param.options.map((o) => ({
              value: o,
              label: o,
            }))}
            bind:value={values[param.name]}
            placeholder={param.description || `Select ${param.name}`}
          />
        {:else if param.type === 'boolean'}
          <div class="flex items-center gap-2">
            <Checkbox
              aria-label={param.name}
              checked={values[param.name] === 'true'}
              onCheckedChange={(checked) => {
                values[param.name] = checked === true ? 'true' : 'false';
              }}
              id={`param-${param.name}`}
            />
            <Label for={`param-${param.name}`} class="text-sm text-muted-foreground">
              {param.description}
            </Label>
          </div>
        {:else}
          <Input bind:value={values[param.name]} placeholder={param.description || param.name} />
        {/if}
      </div>
    </div>
  {/each}
</div>
