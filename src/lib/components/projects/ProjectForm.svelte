<!--
	Unified project form component for both creating and updating projects
	Supports auto-detection of project properties when path is selected
-->

<script lang="ts">
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import FolderPicker from '$lib/components/ui/folder-picker.svelte';
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
    TooltipProvider,
  } from '$lib/components/ui/tooltip';
  import RefreshCw from '~icons/fa6-solid/rotate';
  import Info from '~icons/fa6-solid/circle-info';
  import type { CreateProjectRequest } from '$lib/domains/projects/types';
  import { ProjectFormState } from '$lib/domains/projects/state/projectForm.svelte';
  import ProjectFormNotice from './ProjectFormNotice.svelte';
  import ProjectEntityPickers from './ProjectEntityPickers.svelte';

  interface Props {
    projectId?: number;
    initialData?: Partial<CreateProjectRequest>;
    onSubmit: (data: CreateProjectRequest) => Promise<void>;
    onCancel: () => void;
    isLoading?: boolean;
  }

  let { projectId, initialData = {}, onSubmit, onCancel, isLoading = false }: Props = $props();

  const f = new ProjectFormState(() => ({ projectId, initialData, onSubmit }));
</script>

<div class="space-y-6">
  {#if f.success}
    <ProjectFormNotice kind="success" message={f.success} />
  {/if}
  {#if f.error}
    <ProjectFormNotice kind="error" message={f.error} />
  {/if}

  <form onsubmit={f.handleSubmit} class="space-y-8">
    <!-- Basic Information -->
    <Card>
      <CardHeader>
        <div class="flex items-center justify-between">
          <div>
            <CardTitle>Basic Information</CardTitle>
          </div>
          <TooltipProvider>
            <Tooltip>
              <TooltipTrigger>
                {#snippet child({ props })}
                  <Button
                    {...props}
                    type="button"
                    variant="outline"
                    size="sm"
                    onclick={f.handleSync}
                    disabled={isLoading || f.isAnalyzing || !f.path.trim() || !f.isTauri}
                    class="flex items-center gap-2"
                  >
                    <RefreshCw class="h-4 w-4 {f.isAnalyzing ? 'animate-spin' : ''}" />
                    <span class="hidden sm:inline">Sync</span>
                  </Button>
                {/snippet}
              </TooltipTrigger>
              {#if !f.isTauri}
                <TooltipContent>
                  <p class="max-w-xs">
                    This feature requires the desktop app. Project analysis and auto-detection need
                    access to the file system. Please use the Tauri desktop application to create or
                    update projects with automatic property detection.
                  </p>
                </TooltipContent>
              {:else}
                <TooltipContent>
                  <p>Sync and re-detect project properties</p>
                </TooltipContent>
              {/if}
            </Tooltip>
          </TooltipProvider>
        </div>
      </CardHeader>
      <CardContent class="space-y-6">
        <!-- Project Path - First and most important -->
        <div class="space-y-2">
          <FolderPicker
            bind:value={f.path}
            label="Project Path"
            description={f.isTauri
              ? 'Select the directory where your project will be located'
              : 'Enter the project path manually. File browser is only available in the desktop app.'}
            placeholder="/path/to/your/project"
            disabled={isLoading || f.isAnalyzing}
            required
            onChange={f.handlePathChange}
          />
          {#if !f.isTauri}
            <div
              class="flex items-start gap-2 rounded-md border border-yellow-200 bg-yellow-50 p-3 dark:border-yellow-800 dark:bg-yellow-950"
            >
              <Info class="mt-0.5 h-4 w-4 flex-shrink-0 text-yellow-600 dark:text-yellow-400" />
              <div class="text-sm text-yellow-800 dark:text-yellow-200">
                <p class="mb-1 font-medium">File browser unavailable</p>
                <p>
                  The folder picker requires the desktop app for file system access. You can still
                  enter the project path manually, but automatic project analysis and property
                  detection won't be available.
                </p>
              </div>
            </div>
          {/if}
        </div>

        <div class="space-y-2">
          <Label for="name">Project Name *</Label>
          <Input
            id="name"
            bind:value={f.name}
            placeholder="My Awesome Project"
            disabled={isLoading || f.isAnalyzing}
            required
          />
        </div>

        <div class="space-y-2">
          <Label for="description">Description</Label>
          <Textarea
            id="description"
            bind:value={f.description}
            placeholder="Brief description of your project..."
            disabled={isLoading || f.isAnalyzing}
            rows={3}
          />
        </div>
      </CardContent>
    </Card>

    <!-- Framework, Package Manager & Languages -->
    <Card>
      <CardHeader>
        <CardTitle>Framework, Package Manager & Languages</CardTitle>
      </CardHeader>
      <CardContent class="space-y-6">
        <ProjectEntityPickers
          frameworks={f.frameworks}
          packageManagers={f.packageManagers}
          languages={f.languages}
          bind:frameworkValues={f.frameworkValueStrings}
          bind:packageManagerValues={f.packageManagerValueStrings}
          bind:languageValues={f.languageValueStrings}
          disabled={isLoading || f.isAnalyzing || f.isLoadingEntities}
        />
      </CardContent>
    </Card>

    <!-- Form Actions -->
    <div class="flex justify-end space-x-4">
      <Button
        type="button"
        variant="outline"
        onclick={onCancel}
        disabled={isLoading || f.isAnalyzing}
      >
        Cancel
      </Button>
      <Button type="submit" disabled={isLoading || f.isAnalyzing}>
        {#if f.isAnalyzing}
          Analyzing...
        {:else if isLoading}
          {#if projectId}
            Updating...
          {:else}
            Creating...
          {/if}
        {:else if projectId}
          Update Project
        {:else}
          Create Project
        {/if}
      </Button>
    </div>
  </form>
</div>
