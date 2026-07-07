<script lang="ts">
  import { onMount } from "svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { FolderOpen, FolderPlus } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button";
  import { projectService } from "$lib/domains/projects";
  import type { Project } from "$lib/domains/projects";

  interface Props {
    value: string;
    disabled?: boolean;
  }

  let { value = $bindable(""), disabled = false }: Props = $props();

  let projects = $state<Project[]>([]);
  let loading = $state(false);
  let importPath = $state<string | null>(null);
  let importing = $state(false);
  let importError = $state<string | null>(null);

  const STORAGE_KEY = "portal-coder-last-workspace";

  function loadLastWorkspace(): string | null {
    if (typeof window === "undefined") return null;
    try {
      return localStorage.getItem(STORAGE_KEY);
    } catch {
      return null;
    }
  }

  function saveLastWorkspace(path: string): void {
    if (typeof window === "undefined" || !path) return;
    try {
      localStorage.setItem(STORAGE_KEY, path);
    } catch {
      // ignore
    }
  }

  function resolveDefaultWorkspace(): string {
    const cached = loadLastWorkspace();
    if (cached) return cached;
    return projects[0]?.path ?? "";
  }

  onMount(load);

  $effect(() => {
    if (!disabled && !loading && !value && projects.length > 0) {
      const resolved = resolveDefaultWorkspace();
      if (resolved) value = resolved;
    }
  });

  $effect(() => {
    if (value) saveLastWorkspace(value);
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

  function onSelect(e: Event) {
    value = (e.currentTarget as HTMLSelectElement).value;
  }

  async function browse() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir !== "string") return;
    const match = projects.find((p) => p.path === dir);
    if (match) {
      value = match.path;
      importPath = null;
    } else {
      // Unknown path — offer to import it as a project.
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
      value = project.path;
      importPath = null;
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

<div class="flex flex-1 flex-col gap-1">
  <div class="flex flex-1 items-center gap-1">
    <select
      value={value}
      onchange={onSelect}
      {disabled}
      class="flex-1 rounded border border-border bg-background px-2 py-1 font-mono text-xs disabled:opacity-60"
    >
      <option value="">
        {loading ? "Loading projects…" : "Select a project…"}
      </option>
      {#each projects as p (p.id)}
        <option value={p.path}>{p.name} — {p.path}</option>
      {/each}
      {#if value && !projects.some((p) => p.path === value)}
        <option value={value}>{value}</option>
      {/if}
    </select>

    <Button
      size="icon"
      variant="ghost"
      title="Browse for a folder"
      onclick={browse}
      {disabled}
    >
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
        {importing ? "Importing…" : "Import"}
      </Button>
      <Button size="sm" variant="ghost" onclick={cancelImport} disabled={importing}>
        Cancel
      </Button>
    </div>
  {/if}

  {#if importError}
    <div class="text-xs text-destructive">{importError}</div>
  {/if}
</div>
