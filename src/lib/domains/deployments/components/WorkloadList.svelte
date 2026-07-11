<script lang="ts">
  import type { Deployment, DockerContainer } from "../types";
  import type { GroupByMode, WorkloadGroup, WorkloadSection } from "../utils/workloadGrouping";
  import {
    groupWorkloads,
    sectionLabel,
  } from "../utils/workloadGrouping";
  import WorkloadGroupCard from "./WorkloadGroupCard.svelte";
  import ContainerCard from "./ContainerCard.svelte";
  import Select from "$lib/components/ui/select.svelte";

  type ContainerStatusTab = "running" | "stopped" | "other";

  interface Props {
    containers: DockerContainer[];
    deployments: Deployment[];
    groupBy: GroupByMode;
    statusFilter: ContainerStatusTab;
    searchQuery: string;
    onGroupByChange?: (mode: GroupByMode) => void;
    onStart?: (id: string) => void;
    onStop?: (id: string) => void;
    onRemove?: (id: string) => void;
    onStartDeployment?: (id: string) => void;
    onStopDeployment?: (id: string) => void;
  }

  let {
    containers,
    deployments,
    groupBy,
    statusFilter,
    searchQuery,
    onGroupByChange,
    onStart,
    onStop,
    onRemove,
    onStartDeployment,
    onStopDeployment,
  }: Props = $props();

  const groupByOptions = [
    { value: "stack", label: "Stack / Workload" },
    { value: "image", label: "Image" },
    { value: "network", label: "Network" },
    { value: "flat", label: "Flat list" },
  ];

  let groups = $derived(
    groupWorkloads(containers, deployments, groupBy, {
      statusFilter,
      searchQuery,
    }),
  );

  let sections = $derived.by(() => {
    if (groupBy !== "stack") {
      return [{ section: null as WorkloadSection | null, groups }];
    }

    const sectionMap = new Map<WorkloadSection | "other", WorkloadGroup[]>();
    for (const group of groups) {
      const key = group.section;
      const list = sectionMap.get(key) ?? [];
      list.push(group);
      sectionMap.set(key, list);
    }

    const order: WorkloadSection[] = ["portal", "compose", "standalone"];
    return order
      .filter((section) => sectionMap.has(section))
      .map((section) => ({
        section,
        groups: sectionMap.get(section) ?? [],
      }));
  });
</script>

<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
  <p class="text-sm text-muted-foreground">
    {groups.length} group{groups.length === 1 ? "" : "s"}
    · {containers.length} container{containers.length === 1 ? "" : "s"} total
  </p>
  <Select
    options={groupByOptions}
    value={groupBy}
    placeholder="Group by"
    onSelect={(value) => onGroupByChange?.(value as GroupByMode)}
    class="w-full sm:w-[220px]"
  />
</div>

{#if groups.length === 0}
  <!-- empty state handled by parent -->
{:else if groupBy === "flat"}
  <div class="grid gap-4 md:grid-cols-2">
    {#each groups as group (group.id)}
      {#each group.containers as container (container.id)}
        <ContainerCard
          {container}
          {onStart}
          {onStop}
          {onRemove}
        />
      {/each}
    {/each}
  </div>
{:else}
  <div class="space-y-6">
    {#each sections as { section, groups: sectionGroups }}
      {#if section}
        <div class="space-y-3">
          <h3 class="text-sm font-medium tracking-wide text-muted-foreground uppercase">
            {sectionLabel(section)}
          </h3>
          <div class="space-y-4">
            {#each sectionGroups as group (group.id)}
              <WorkloadGroupCard
                {group}
                defaultOpen={group.containers.length <= 4}
                {onStart}
                {onStop}
                {onRemove}
                {onStartDeployment}
                {onStopDeployment}
              />
            {/each}
          </div>
        </div>
      {:else}
        <div class="space-y-4">
          {#each sectionGroups as group (group.id)}
            <WorkloadGroupCard
              {group}
              defaultOpen={group.containers.length <= 4}
              {onStart}
              {onStop}
              {onRemove}
              {onStartDeployment}
              {onStopDeployment}
            />
          {/each}
        </div>
      {/if}
    {/each}
  </div>
{/if}
