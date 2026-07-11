<!--
	Parameter Editor Component
	Handles editing a single script parameter
-->

<script lang="ts">
  import Select from "$lib/components/ui/select.svelte";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Button } from "$lib/components/ui/button";
  import { Checkbox } from "$lib/components/ui/checkbox";
  import { X } from "@lucide/svelte";
  import { FilePicker } from "$lib/domains/shared/components";
  import type { ScriptParameter } from "$lib/domains/custom_scripts/services/customScriptService";

  interface Props {
    parameter: ScriptParameter;
    index: number;
    onUpdate: (index: number, updates: Partial<ScriptParameter>) => void;
    onRemove: (index: number) => void;
  }

  let { parameter, index, onUpdate, onRemove }: Props = $props();
</script>

<div class="space-y-4 rounded-lg border p-4">
  <div class="flex items-center justify-between">
    <h4 class="font-medium">Parameter {index + 1}</h4>
    <Button
      type="button"
      variant="ghost"
      size="sm"
      onclick={() => onRemove(index)}
    >
      <X class="h-4 w-4" />
    </Button>
  </div>

  <div class="grid grid-cols-2 gap-4">
    <div>
      <Label>Name</Label>
      <Input
        value={parameter.name}
        oninput={(e) =>
          onUpdate(index, { name: (e.target as HTMLInputElement).value })}
        placeholder="VPN_CONFIG"
      />
    </div>

    <div>
      <Label>Label</Label>
      <Input
        value={parameter.label}
        oninput={(e) =>
          onUpdate(index, { label: (e.target as HTMLInputElement).value })}
        placeholder="VPN Directory"
      />
    </div>
  </div>

  <div>
    <Label>Type</Label>
    <Select
      defaultValue={parameter.parameter_type}
      options={[
        { value: "string", label: "String" },
        { value: "file", label: "File" },
        { value: "folder", label: "Folder" },
        { value: "number", label: "Number" },
        { value: "boolean", label: "Boolean" },
        { value: "password", label: "Password" },
      ]}
      onSelect={(value) =>
        onUpdate(index, {
          parameter_type: value as ScriptParameter["parameter_type"],
        })}
    />
  </div>

  {#if parameter.parameter_type === "file"}
    <div>
      <Label>File Filters (comma-separated)</Label>
      <Input
        value={parameter.file_filters?.join(", ") || ""}
        oninput={(e) => {
          const filters = (e.target as HTMLInputElement).value
            .split(",")
            .map((f) => f.trim())
            .filter(Boolean);
          onUpdate(index, {
            file_filters: filters.length > 0 ? filters : undefined,
          });
        }}
        placeholder="e.g., *.ovpn, *.conf"
      />
    </div>
  {/if}

  <div>
    <Label>Description</Label>
    <Input
      value={parameter.description || ""}
      oninput={(e) =>
        onUpdate(index, {
          description: (e.target as HTMLInputElement).value || undefined,
        })}
      placeholder="Optional description"
    />
  </div>

  <div>
    <Label>Default Value</Label>
    {#if parameter.parameter_type === "file"}
      {@const fileFilters = parameter.file_filters || []}
      <FilePicker
        value={parameter.default_value || ""}
        label=""
        description=""
        filters={fileFilters.length > 0
          ? [{ name: "Files", extensions: fileFilters }]
          : []}
        selectFolder={false}
        onChange={(path) => {
          onUpdate(index, {
            default_value: path || undefined,
          });
        }}
      />
    {:else if parameter.parameter_type === "folder"}
      <FilePicker
        value={parameter.default_value || ""}
        label=""
        description=""
        selectFolder={true}
        onChange={(path) => {
          onUpdate(index, {
            default_value: path || undefined,
          });
        }}
      />
    {:else}
      <Input
        value={parameter.default_value || ""}
        oninput={(e) =>
          onUpdate(index, {
            default_value: (e.target as HTMLInputElement).value || undefined,
          })}
        placeholder="Optional default value"
      />
    {/if}
  </div>

  <div class="flex items-center gap-2">
    <Checkbox
      checked={parameter.required}
      onCheckedChange={(checked) =>
        onUpdate(index, { required: checked === true })}
    />
    <Label>Required</Label>
  </div>
</div>
