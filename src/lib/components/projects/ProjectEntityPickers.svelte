<script lang="ts">
  import { Label } from '$lib/components/ui/label';
  import MultiSelect from '$lib/components/ui/multi-select.svelte';

  type Entity = { id: number; name: string; icon?: string; icon_type?: string };

  interface Props {
    frameworks: Entity[];
    packageManagers: Entity[];
    languages: Entity[];
    frameworkValues: string[];
    packageManagerValues: string[];
    languageValues: string[];
    disabled: boolean;
  }
  let {
    frameworks,
    packageManagers,
    languages,
    frameworkValues = $bindable(),
    packageManagerValues = $bindable(),
    languageValues = $bindable(),
    disabled,
  }: Props = $props();

  const toOptions = (list: Entity[]) =>
    list.map((e) => ({
      value: e.id.toString(),
      label: e.name,
      icon: e.icon,
      iconType: e.icon_type,
    }));
</script>

<div class="grid grid-cols-1 gap-6 md:grid-cols-3">
  <div class="space-y-2">
    <Label for="frameworks">Frameworks</Label>
    <MultiSelect
      options={toOptions(frameworks)}
      bind:value={frameworkValues}
      placeholder="Select frameworks..."
      searchPlaceholder="Search frameworks..."
      {disabled}
      class="w-full"
    />
  </div>
  <div class="space-y-2">
    <Label for="package-managers">Package Managers</Label>
    <MultiSelect
      options={toOptions(packageManagers)}
      bind:value={packageManagerValues}
      placeholder="Select package managers..."
      searchPlaceholder="Search package managers..."
      {disabled}
      class="w-full"
    />
  </div>
  <div class="space-y-2">
    <Label for="languages">Languages</Label>
    <MultiSelect
      options={toOptions(languages)}
      bind:value={languageValues}
      placeholder="Select languages..."
      searchPlaceholder="Search languages..."
      {disabled}
      class="w-full"
    />
  </div>
</div>
