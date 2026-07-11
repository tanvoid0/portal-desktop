<script lang="ts">
  import { cn } from "$lib/utils";
  import * as Select from "$lib/components/ui/select";
  import ShellIcon from "./ShellIcon.svelte";

  export interface ShellProfileOption {
    name: string;
    command: string;
    icon: string;
  }

  interface Props {
    profiles: ShellProfileOption[];
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    onSelect?: (value: string) => void;
    class?: string;
  }

  let {
    profiles,
    value = $bindable(""),
    placeholder = "Select shell...",
    disabled = false,
    onSelect,
    class: className = "",
  }: Props = $props();

  const selectedProfile = $derived(
    profiles.find((profile) => profile.name === value),
  );

  function handleValueChange(newValue: string | undefined) {
    const selected = newValue ?? "";
    value = selected;
    onSelect?.(selected);
  }
</script>

<Select.Root
  type="single"
  bind:value
  onValueChange={handleValueChange}
  {disabled}
>
  <Select.Trigger
    size="sm"
    class={cn("min-w-[10rem] max-w-[14rem] bg-muted text-xs text-foreground", className)}
  >
    <span
      data-slot="select-value"
      class="flex min-w-0 items-center gap-1.5 truncate"
    >
      {#if selectedProfile}
        <ShellIcon icon={selectedProfile.icon} size="xs" />
        <span class="truncate">{selectedProfile.name}</span>
      {:else}
        <span class="text-muted-foreground">{placeholder}</span>
      {/if}
    </span>
  </Select.Trigger>
  <Select.Content class="min-w-[var(--bits-select-anchor-width)]">
    {#each profiles as profile (profile.name)}
      <Select.Item value={profile.name} label={profile.name}>
        {#snippet children()}
          <ShellIcon icon={profile.icon} size="xs" />
          <span class="truncate">{profile.name}</span>
        {/snippet}
      </Select.Item>
    {/each}
  </Select.Content>
</Select.Root>
