<!--
	ProjectCard component for displaying project information
	Uses Svelte 5 runes and Tailwind CSS
-->

<script lang="ts">
  import { onMount } from "svelte";
  import type { Project } from "$lib/domains/projects/types";
  import { formatRelativeTime, formatBytes } from "$lib/domains/shared/utils";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import TechIcon from "$lib/components/ui/tech-icon.svelte";
  import {
    Edit,
    Trash2,
    Star,
    FolderOpen,
    HardDrive,
    FileText,
    Clock,
    Eye,
    Calendar,
    GitBranch,
    GitCommit,
    CircleDot,
    Globe,
    Archive,
    CheckCircle2,
  } from "@lucide/svelte";
  import {
    getProjectGitBranch,
    getProjectGitCommit,
    getProjectFileCount,
    getProjectSize,
    getFrameworkColor,
    getFrameworkIconBackground,
    getStatusColor,
    truncatePath,
    shortCommitHash,
  } from "$lib/domains/projects/utils/display";
  import {
    projectIconRegistry,
    type ResolvedTechIcon,
  } from "$lib/domains/projects/utils/iconRegistry";

  interface Props {
    project: Project;
    onClick?: (project: Project) => void;
    onEdit?: (project: Project) => void;
    onDelete?: (project: Project) => void;
    showActions?: boolean;
  }

  let {
    project,
    onClick = () => {},
    onEdit = () => {},
    onDelete = () => {},
    showActions = true,
  }: Props = $props();

  let registryReady = $state(false);

  onMount(async () => {
    await projectIconRegistry.ensureLoaded();
    registryReady = true;
  });

  const primaryFramework = $derived(
    projectIconRegistry.resolvePrimaryFramework(project),
  );
  const frameworks = $derived(projectIconRegistry.resolveFrameworks(project));
  const packageManagers = $derived(
    projectIconRegistry.resolvePackageManagers(project),
  );
  const languages = $derived(projectIconRegistry.resolveLanguages(project));
  const showTechStack = $derived(
    registryReady &&
      (frameworks.length > 0 ||
        packageManagers.length > 0 ||
        languages.length > 0),
  );
  const gitBranch = $derived(getProjectGitBranch(project));
  const gitCommit = $derived(shortCommitHash(getProjectGitCommit(project)));
  const fileCount = $derived(getProjectFileCount(project));
  const projectSize = $derived(getProjectSize(project));
  const hasUncommittedChanges = $derived(
    project.metadata?.gitInfo?.hasUncommittedChanges ??
      project.has_uncommitted_changes,
  );
  const outdatedCount = $derived(
    project.metadata?.dependencies?.outdated?.length ?? 0,
  );
  const vulnerabilityCount = $derived(
    project.metadata?.dependencies?.vulnerabilities?.length ?? 0,
  );

  const handleClick = () => {
    onClick(project);
  };

  const handleEdit = (e: Event) => {
    e.stopPropagation();
    onEdit(project);
  };

  const handleDelete = (e: Event) => {
    e.stopPropagation();
    onDelete(project);
  };

  const statusIcon = $derived(
    project.status === "active"
      ? CheckCircle2
      : project.status === "archived"
        ? Archive
        : null,
  );

  function techBadgeClass(item: ResolvedTechIcon): string {
    return getFrameworkColor(item.name);
  }
</script>

<Card
  class="group h-full cursor-pointer transition-shadow hover:shadow-md"
  onclick={handleClick}
  role="button"
  tabindex={0}
  onkeydown={(e) => e.key === "Enter" && handleClick()}
>
  <CardHeader class="pb-3">
    <div class="flex w-full items-start justify-between gap-3">
      <div class="flex min-w-0 flex-1 items-start gap-3">
        <div
          class="flex h-11 w-11 shrink-0 items-center justify-center rounded-lg {getFrameworkIconBackground(
            primaryFramework?.name,
          )}"
        >
          {#if primaryFramework}
            <TechIcon
              icon={primaryFramework.icon}
              iconType={primaryFramework.icon_type}
              size="lg"
              alt={primaryFramework.name}
            />
          {:else}
            <FolderOpen class="h-6 w-6 text-muted-foreground" />
          {/if}
        </div>
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-1.5">
            <CardTitle class="truncate">{project.name}</CardTitle>
            {#if project.starred}
              <Star
                class="h-4 w-4 shrink-0 fill-yellow-400 text-yellow-400"
                aria-label="Starred project"
              />
            {/if}
          </div>
          {#if project.description}
            <CardDescription class="mt-1 line-clamp-2">
              {project.description}
            </CardDescription>
          {/if}
        </div>
      </div>

      <Badge
        variant="outline"
        class="shrink-0 gap-1 capitalize {getStatusColor(project.status)}"
      >
        {#if statusIcon}
          {@const StatusIcon = statusIcon}
          <StatusIcon class="h-3 w-3" />
        {/if}
        {project.status}
      </Badge>
    </div>
  </CardHeader>

  <CardContent>
    <div class="space-y-3">
      {#if showTechStack}
        <div class="flex flex-wrap items-center gap-1.5">
          {#each frameworks as item (item.name)}
            <Badge variant="outline" class="gap-1.5 {techBadgeClass(item)}">
              <TechIcon
                icon={item.icon}
                iconType={item.icon_type}
                size="xs"
                alt={item.name}
              />
              {item.name}
            </Badge>
          {/each}

          {#each packageManagers as item (item.name)}
            <Badge variant="outline" class="gap-1.5">
              <TechIcon
                icon={item.icon}
                iconType={item.icon_type}
                size="xs"
                alt={item.name}
              />
              {item.name}
            </Badge>
          {/each}

          {#each languages as item (item.name)}
            <Badge variant="secondary" class="gap-1.5">
              <TechIcon
                icon={item.icon}
                iconType={item.icon_type}
                size="xs"
                alt={item.name}
              />
              {item.name}
            </Badge>
          {/each}
        </div>
      {/if}

      <div class="flex flex-wrap items-center gap-1.5">
        {#if gitBranch}
          <Badge variant="outline" class="gap-1">
            <GitBranch class="h-3 w-3" />
            {gitBranch}
          </Badge>
        {/if}

        {#if gitCommit}
          <Badge variant="secondary" class="gap-1 font-mono text-xs">
            <GitCommit class="h-3 w-3" />
            {gitCommit}
          </Badge>
        {/if}

        {#if hasUncommittedChanges}
          <Badge variant="outline" class="gap-1 text-amber-700 dark:text-amber-300">
            <CircleDot class="h-3 w-3" />
            Uncommitted
          </Badge>
        {/if}
      </div>

      <div
        class="flex items-center gap-1.5 text-xs text-muted-foreground"
        title={project.path}
      >
        <FolderOpen class="h-3.5 w-3.5 shrink-0" />
        <span class="truncate font-mono">{truncatePath(project.path)}</span>
      </div>

      <div class="grid grid-cols-2 gap-x-3 gap-y-1.5 text-xs text-muted-foreground">
        <div class="flex items-center gap-1.5">
          <HardDrive class="h-3.5 w-3.5 shrink-0" />
          <span>{formatBytes(projectSize)}</span>
        </div>
        <div class="flex items-center gap-1.5">
          <FileText class="h-3.5 w-3.5 shrink-0" />
          <span>{fileCount} files</span>
        </div>
        {#if project.last_opened}
          <div class="flex items-center gap-1.5">
            <Clock class="h-3.5 w-3.5 shrink-0" />
            <span>{formatRelativeTime(project.last_opened)}</span>
          </div>
        {/if}
        {#if project.open_count > 0}
          <div class="flex items-center gap-1.5">
            <Eye class="h-3.5 w-3.5 shrink-0" />
            <span>{project.open_count} opens</span>
          </div>
        {/if}
        {#if project.dev_port}
          <div class="flex items-center gap-1.5">
            <Globe class="h-3.5 w-3.5 shrink-0" />
            <span>:{project.dev_port}</span>
          </div>
        {/if}
        {#if project.created_at}
          <div class="flex items-center gap-1.5">
            <Calendar class="h-3.5 w-3.5 shrink-0" />
            <span>Created {formatRelativeTime(project.created_at)}</span>
          </div>
        {/if}
      </div>

      {#if outdatedCount > 0 || vulnerabilityCount > 0}
        <div class="flex flex-wrap gap-1.5">
          {#if outdatedCount > 0}
            <Badge variant="outline" class="text-xs text-amber-700 dark:text-amber-300">
              {outdatedCount} outdated
            </Badge>
          {/if}
          {#if vulnerabilityCount > 0}
            <Badge variant="destructive" class="text-xs">
              {vulnerabilityCount} vulnerable
            </Badge>
          {/if}
        </div>
      {/if}

      {#if showActions}
        <div
          class="divider-edge-t divider-edge-full flex items-center gap-1 pt-2 opacity-0 transition-opacity group-hover:opacity-100"
        >
          <Button
            variant="ghost"
            size="sm"
            onclick={handleEdit}
            class="h-8 w-8 p-0"
            title="Edit project"
            aria-label="Edit project"
          >
            <Edit class="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            onclick={handleDelete}
            class="h-8 w-8 p-0 text-red-600 hover:text-red-700 dark:text-red-400 dark:hover:text-red-300"
            title="Delete project"
            aria-label="Delete project"
          >
            <Trash2 class="h-4 w-4" />
          </Button>
        </div>
      {/if}
    </div>
  </CardContent>
</Card>
