<script lang="ts">
  import { onMount } from 'svelte';
  import { open as openFolderDialog } from '@tauri-apps/plugin-dialog';
  import { cn } from '$lib/utils';
  import * as Popover from '$lib/components/ui/popover';
  import * as Command from '$lib/components/ui/command';
  import { Button } from '$lib/components/ui/button';
  import Check from '~icons/fa6-solid/check';
  import ChevronsUpDown from '~icons/fa6-solid/up-down';
  import FolderOpen from '~icons/fa6-solid/folder-open';
  import FolderPlus from '~icons/fa6-solid/folder-plus';
  import { projectService } from '$lib/domains/projects/services/projectService';
  import type { Project } from '$lib/domains/projects/types';

  interface Props {
    value: string;
    disabled?: boolean;
    placeholder?: string;
    /** localStorage key used to remember + default to the last pick. Omit to skip persistence. */
    storageKey?: string;
    onSelect?: (path: string, projectId?: string) => void;
  }

  let {
    value = $bindable(''),
    disabled = false,
    placeholder = 'Select a project…',
    storageKey,
    onSelect,
  }: Props = $props();

  let projects = $state<Project[]>([]);
  let loading = $state(false);
  let open = $state(false);
  let search = $state('');
  let importPath = $state<string | null>(null);
  let importing = $state(false);
  let importError = $state<string | null>(null);

  function loadLast(): string | null {
    if (!storageKey || typeof window === 'undefined') return null;
    try {
      return localStorage.getItem(storageKey);
    } catch {
      return null;
    }
  }

  function saveLast(path: string): void {
    if (!storageKey || typeof window === 'undefined' || !path) return;
    try {
      localStorage.setItem(storageKey, path);
    } catch {
      // ignore
    }
  }

  onMount(load);

  // Default to the last-used path (or the first known project) once projects load.
  $effect(() => {
    if (!disabled && !loading && !value && projects.length > 0) {
      const resolved = loadLast() ?? projects[0]?.path ?? '';
      if (resolved) value = resolved;
    }
  });

  $effect(() => {
    if (value) saveLast(value);
  });

  $effect(() => {
    if (!open) search = '';
  });

  async function load() {
    loading = true;
    try {
      projects = await projectService.loadProjects();
    } catch {
      projects = [];
    } finally {
      loading = false;
    }
  }

  function basename(p: string): string {
    const parts = p.split(/[/\\]/).filter(Boolean);
    return parts[parts.length - 1] ?? p;
  }

  const selectedProject = $derived(projects.find((p) => p.path === value) ?? null);

  const filteredProjects = $derived.by(() => {
    const q = search.trim().toLowerCase();
    if (!q) return projects;
    return projects.filter(
      (p) => p.name.toLowerCase().includes(q) || p.path.toLowerCase().includes(q)
    );
  });

  function handleSelect(project: Project) {
    value = project.path;
    onSelect?.(project.path, project.id);
    open = false;
    search = '';
  }

  async function browse() {
    const dir = await openFolderDialog({ directory: true, multiple: false });
    if (typeof dir !== 'string') return;
    const match = projects.find((p) => p.path === dir);
    if (match) {
      handleSelect(match);
      importPath = null;
    } else {
      // Unknown path — let it through as free text, and offer to import it.
      value = dir;
      onSelect?.(dir);
      importPath = dir;
      importError = null;
    }
  }

  async function confirmImport() {
    if (!importPath) return;
    importing = true;
    importError = null;
    try {
      const project = await projectService.createProject({
        name: basename(importPath),
        path: importPath,
      });
      projects = [...projects, project];
      handleSelect(project);
    } catch (e) {
      importError = String(e);
    } finally {
      importing = false;
    }
  }

  function cancelImport() {
    importPath = null;
    importError = null;
  }
</script>

<div class="flex min-w-0 flex-col gap-1.5">
  <div class="flex min-w-0 items-center gap-2">
    <div class="min-w-0 flex-1">
      <Popover.Root bind:open>
        <Popover.Trigger class="w-full">
          <Button
            variant="outline"
            role="combobox"
            aria-expanded={open}
            disabled={disabled || loading}
            class={cn(
              'w-full justify-between px-3 font-normal',
              !value && 'text-muted-foreground',
              open && 'border-ring'
            )}
          >
            <span class="min-w-0 flex-1 truncate text-left font-mono text-xs">
              {#if loading}
                Loading projects…
              {:else if selectedProject}
                {selectedProject.name} — {selectedProject.path}
              {:else if value}
                {value}
              {:else}
                {placeholder}
              {/if}
            </span>
            <ChevronsUpDown
              class={cn(
                'ms-2 size-4 shrink-0 opacity-50 transition-transform duration-200',
                open && 'rotate-180'
              )}
            />
          </Button>
        </Popover.Trigger>

        <Popover.Content
          class="w-[min(100vw-2rem,32rem)] p-0 shadow-md"
          align="start"
          sideOffset={4}
        >
          <Command.Root shouldFilter={false} class="rounded-lg">
            <Command.Input placeholder="Type to filter projects…" bind:value={search} class="h-9" />
            <Command.Empty class="py-6 text-center text-sm text-muted-foreground">
              No projects found.
            </Command.Empty>
            <Command.List class="max-h-72 overflow-y-auto p-1">
              <Command.Group heading="Projects">
                {#each filteredProjects as project (project.id)}
                  <Command.Item
                    value={project.path}
                    onclick={() => handleSelect(project)}
                    class={cn(
                      'relative flex cursor-pointer items-start rounded-md px-2 py-2.5 transition-colors outline-none select-none',
                      'aria-selected:bg-accent aria-selected:text-accent-foreground',
                      'hover:bg-accent hover:text-accent-foreground',
                      value === project.path && 'bg-accent/50'
                    )}
                  >
                    <Check
                      class={cn(
                        'me-2 mt-0.5 size-4 shrink-0 transition-opacity',
                        value === project.path ? 'text-primary opacity-100' : 'opacity-0'
                      )}
                    />
                    <div class="min-w-0 flex-1">
                      <div class="truncate text-sm">{project.name}</div>
                      <div class="truncate text-xs text-muted-foreground">{project.path}</div>
                    </div>
                  </Command.Item>
                {/each}
              </Command.Group>
            </Command.List>
          </Command.Root>
        </Popover.Content>
      </Popover.Root>
    </div>

    <Button size="icon" variant="ghost" title="Browse for a folder" onclick={browse} {disabled}>
      <FolderOpen class="h-4 w-4" />
    </Button>
  </div>

  {#if importPath}
    <div
      class="flex items-center gap-2 rounded border border-border bg-muted/40 px-2 py-1.5 text-xs"
    >
      <FolderPlus class="h-3.5 w-3.5 shrink-0 text-primary" />
      <span class="min-w-0 flex-1 truncate">
        Import <span class="font-mono">{importPath}</span> as a new project?
      </span>
      <Button size="sm" onclick={confirmImport} disabled={importing}>
        {importing ? 'Importing…' : 'Import'}
      </Button>
      <Button size="sm" variant="ghost" onclick={cancelImport} disabled={importing}>Cancel</Button>
    </div>
  {/if}

  {#if importError}
    <div class="text-xs text-destructive">{importError}</div>
  {/if}
</div>
