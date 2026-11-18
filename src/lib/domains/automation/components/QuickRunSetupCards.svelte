<!-- Working directory + package manager picker, and one-click presets. -->
<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Label } from '$lib/components/ui/label';
  import FilePicker from '$lib/components/ui/file-picker.svelte';
  import Select from '$lib/components/ui/select.svelte';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import Zap from '~icons/fa6-solid/bolt';
  import SectionCard from '$lib/components/shared/SectionCard.svelte';
  import { automation, type AutomationStepRef } from '$lib/domains/automation';
  import type { Project } from '$lib/domains/projects/types';

  interface Props {
    cwd: string;
    packageManager: string;
    projects: Project[];
    selectedProjectId: string;
    onCwdChange: (path: string) => void;
    onProjectSelect: (projectId: string) => void;
    onPreset: (preset: AutomationStepRef[]) => void;
  }

  let {
    cwd,
    packageManager = $bindable(),
    projects,
    selectedProjectId,
    onCwdChange,
    onProjectSelect,
    onPreset,
  }: Props = $props();

  const presets = $derived([
    { label: 'Install', steps: () => automation.presets.install(packageManager) },
    { label: 'Test', steps: () => automation.presets.test() },
    { label: 'Build', steps: () => automation.presets.build() },
    { label: 'Full CI', steps: () => automation.presets.ci(packageManager) },
  ]);
</script>

<SectionCard
  title="Working Directory"
  description="Any project folder or directory on disk"
  icon={FolderOpen}
  contentClass="space-y-4"
>
  <FilePicker
    label="Directory"
    value={cwd}
    selectFolder={true}
    placeholder="D:/projects/my-app"
    onChange={onCwdChange}
  />

  {#if projects.length > 0}
    <div class="space-y-2">
      <Label>Or pick a registered project</Label>
      <Select
        options={[
          { value: '', label: '— Custom path —' },
          ...projects.map((p) => ({
            value: p.id,
            label: p.name,
          })),
        ]}
        value={selectedProjectId}
        placeholder="Select project"
        onValueChange={onProjectSelect}
      />
    </div>
  {/if}

  <div class="space-y-2">
    <Label for="package-manager">Package manager</Label>
    <Select
      id="package-manager"
      options={[
        { value: 'npm', label: 'npm' },
        { value: 'pnpm', label: 'pnpm' },
        { value: 'yarn', label: 'yarn' },
        { value: 'bun', label: 'bun' },
      ]}
      bind:value={packageManager}
    />
  </div>
</SectionCard>

<SectionCard
  title="Presets"
  description="Common sequences — customize via queue after applying"
  icon={Zap}
  contentClass="flex flex-wrap gap-2"
>
  {#each presets as preset (preset.label)}
    <Button variant="outline" size="sm" onclick={() => onPreset(preset.steps())}>
      {preset.label}
    </Button>
  {/each}
</SectionCard>
