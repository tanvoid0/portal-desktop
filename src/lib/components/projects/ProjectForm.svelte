<!--
	Unified project form component for both creating and updating projects
	Supports auto-detection of project properties when path is selected
-->

<script lang="ts">
  import { onMount } from "svelte";
  import { invokeClient } from "$lib/utils/invokeClient";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Textarea } from "$lib/components/ui/textarea";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import FolderPicker from "$lib/components/ui/folder-picker.svelte";
  import MultiSelect from "$lib/components/ui/multi-select.svelte";
  import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
    TooltipProvider,
  } from "$lib/components/ui/tooltip";
  import { RefreshCw, Info } from "@lucide/svelte";
  import { logger } from "$lib/domains/shared/services/logger";
  import { isTauriEnvironment } from "$lib/utils/tauri";
  import type { CreateProjectRequest } from "$lib/domains/projects/types";
  import { ideService, type Framework } from "$lib/domains/ide";
  import {
    packageManagerService,
    type PackageManager,
  } from "$lib/domains/package_managers";
  import { languageService, type Language } from "$lib/domains/languages";

  interface ProjectAnalysis {
    name: string;
    frameworks: string[]; // Multiple frameworks detected
    languages: string[]; // Multiple languages detected
    package_managers: string[]; // Multiple package managers detected
    build_command: string | null;
    start_command: string | null;
    test_command: string | null;
    output_directory: string | null;
    dev_port: number | null;
    prod_port: number | null;
  }

  interface Props {
    projectId?: number;
    initialData?: Partial<CreateProjectRequest>;
    onSubmit: (data: CreateProjectRequest) => Promise<void>;
    onCancel: () => void;
    isLoading?: boolean;
  }

  let {
    projectId,
    initialData = {},
    onSubmit,
    onCancel,
    isLoading = false,
  }: Props = $props();

  const log = logger.createScoped("ProjectForm");

  // Check if running in Tauri environment (backend-dependent features require this)
  const isTauri = isTauriEnvironment();

  // Form state
  let name = $state(initialData.name || "");
  let description = $state(initialData.description || "");
  let path = $state(initialData.path || "");
  let frameworkValueStrings = $state<string[]>(
    (initialData.framework_ids || []).map((id) => id.toString()),
  );
  let frameworkIds = $derived(
    frameworkValueStrings
      .map((v) => parseInt(v, 10))
      .filter((id) => !isNaN(id)),
  );
  let packageManagerValueStrings = $state<string[]>(
    (initialData.package_manager_ids || []).map((id) => id.toString()),
  );
  let packageManagerIds = $derived(
    packageManagerValueStrings
      .map((v) => parseInt(v, 10))
      .filter((id) => !isNaN(id)),
  );
  let languageValueStrings = $state<string[]>(
    (initialData.language_ids || []).map((id) => id.toString()),
  );
  let languageIds = $derived(
    languageValueStrings
      .map((v) => parseInt(v, 10))
      .filter((id) => !isNaN(id)),
  );

  // Entity lists
  let frameworks = $state<Framework[]>([]);
  let packageManagers = $state<PackageManager[]>([]);
  let languages = $state<Language[]>([]);
  let isLoadingEntities = $state(false);

  let error = $state("");
  let success = $state("");
  let isAnalyzing = $state(false);

  let detectedCommands = $state({
    build_command: initialData.build_command as string | undefined,
    start_command: initialData.start_command as string | undefined,
    test_command: initialData.test_command as string | undefined,
    output_directory: initialData.output_directory as string | undefined,
    dev_port: initialData.dev_port as number | undefined,
    prod_port: initialData.prod_port as number | undefined,
  });

  // Set up breadcrumbs on mount
  onMount(async () => {
    if (projectId) {
      // This is an update form
      log.info("Initializing project update form", { projectId });
    } else {
      // This is a create form
      log.info("Initializing project create form");
    }
    await loadEntities();
  });

  async function loadEntities() {
    isLoadingEntities = true;
    try {
      await Promise.all([
        loadFrameworks(),
        loadPackageManagers(),
        loadLanguages(),
      ]);
    } catch (error) {
      log.warn("Failed to load entities", { error });
    } finally {
      isLoadingEntities = false;
    }
  }

  async function loadFrameworks() {
    try {
      frameworks = await ideService.getAllFrameworks();
    } catch (error) {
      log.warn("Failed to load frameworks", { error });
    }
  }

  async function loadPackageManagers() {
    try {
      packageManagers = await packageManagerService.getAllPackageManagers();
    } catch (error) {
      log.warn("Failed to load package managers", { error });
    }
  }

  async function loadLanguages() {
    try {
      languages = await languageService.getAllLanguages();
    } catch (error) {
      log.warn("Failed to load languages", { error });
    }
  }

  async function handlePathChange(newPath: string) {
    path = newPath;

    if (newPath.trim()) {
      await analyzeProject(newPath);
    }
  }

  async function analyzeProject(projectPath: string, forceSync = false) {
    try {
      isAnalyzing = true;
      error = "";

      log.info("Analyzing project directory", { path: projectPath, forceSync });

      const analysis: ProjectAnalysis =
        await invokeClient.post<ProjectAnalysis>("analyze_project_directory", {
          path: projectPath,
        });

      log.info("Project analysis completed", analysis);

      // Auto-populate form fields
      if (forceSync || !name.trim()) {
        name = analysis.name;
      }

      // Always update these fields when syncing
      if (forceSync) {
        // Match all detected frameworks by name
        const matchedFrameworkIds: number[] = [];
        for (const frameworkName of analysis.frameworks) {
          const found = frameworks.find(
            (f) => f.name.toLowerCase() === frameworkName.toLowerCase(),
          );
          if (found && !matchedFrameworkIds.includes(found.id)) {
            matchedFrameworkIds.push(found.id);
          }
        }
        frameworkValueStrings = matchedFrameworkIds.map((id) => id.toString());

        // Match all detected package managers by name
        const matchedPMIds: number[] = [];
        for (const pmName of analysis.package_managers) {
          const found = packageManagers.find(
            (pm) => pm.name.toLowerCase() === pmName.toLowerCase(),
          );
          if (found && !matchedPMIds.includes(found.id)) {
            matchedPMIds.push(found.id);
          }
        }
        packageManagerValueStrings = matchedPMIds.map((id) => id.toString());

        // Match all detected languages by name
        const matchedLanguageIds: number[] = [];
        for (const languageName of analysis.languages) {
          const found = languages.find(
            (l) => l.name.toLowerCase() === languageName.toLowerCase(),
          );
          if (found && !matchedLanguageIds.includes(found.id)) {
            matchedLanguageIds.push(found.id);
          }
        }
        languageValueStrings = matchedLanguageIds.map((id) => id.toString());
      } else {
        // Only update if not already set
        if (frameworkValueStrings.length === 0 && analysis.frameworks.length > 0) {
          const matchedFrameworkIds: number[] = [];
          for (const frameworkName of analysis.frameworks) {
            const found = frameworks.find(
              (f) => f.name.toLowerCase() === frameworkName.toLowerCase(),
            );
            if (found && !matchedFrameworkIds.includes(found.id)) {
              matchedFrameworkIds.push(found.id);
            }
          }
          frameworkValueStrings = matchedFrameworkIds.map((id) => id.toString());
        }
        if (
          packageManagerValueStrings.length === 0 &&
          analysis.package_managers.length > 0
        ) {
          const matchedPMIds: number[] = [];
          for (const pmName of analysis.package_managers) {
            const found = packageManagers.find(
              (pm) => pm.name.toLowerCase() === pmName.toLowerCase(),
            );
            if (found && !matchedPMIds.includes(found.id)) {
              matchedPMIds.push(found.id);
            }
          }
          packageManagerValueStrings = matchedPMIds.map((id) => id.toString());
        }
        if (languageValueStrings.length === 0 && analysis.languages.length > 0) {
          const matchedLanguageIds: number[] = [];
          for (const languageName of analysis.languages) {
            const found = languages.find(
              (l) => l.name.toLowerCase() === languageName.toLowerCase(),
            );
            if (found && !matchedLanguageIds.includes(found.id)) {
              matchedLanguageIds.push(found.id);
            }
          }
          languageValueStrings = matchedLanguageIds.map((id) => id.toString());
        }
      }

      detectedCommands = {
        build_command: analysis.build_command ?? undefined,
        start_command: analysis.start_command ?? undefined,
        test_command: analysis.test_command ?? undefined,
        output_directory: analysis.output_directory ?? undefined,
        dev_port: analysis.dev_port ?? undefined,
        prod_port: analysis.prod_port ?? undefined,
      };

      success = forceSync
        ? "Project properties synced successfully!"
        : "Project properties auto-detected successfully!";

      // Clear success message after 3 seconds
      setTimeout(() => {
        success = "";
      }, 3000);
    } catch (err) {
      log.error("Failed to analyze project", err);
      error =
        "Failed to analyze project directory. Please check the path and try again.";
    } finally {
      isAnalyzing = false;
    }
  }

  async function handleSync() {
    if (path.trim()) {
      await analyzeProject(path, true);
    } else {
      error = "Please select a project path first.";
    }
  }

  async function handleSubmit(event: SubmitEvent) {
    event.preventDefault();

    if (!name.trim() || !path.trim()) {
      error = "Please fill in all required fields.";
      return;
    }

    try {
      error = "";
      success = "";

      const projectData: CreateProjectRequest = {
        name: name.trim(),
        description: description.trim() || undefined,
        path: path.trim(),
        framework_ids: frameworkIds,
        package_manager_ids: packageManagerIds,
        language_ids: languageIds,
        build_command: detectedCommands.build_command,
        start_command: detectedCommands.start_command,
        test_command: detectedCommands.test_command,
        output_directory: detectedCommands.output_directory,
        dev_port: detectedCommands.dev_port,
        prod_port: detectedCommands.prod_port,
      };

      await onSubmit(projectData);
    } catch (err) {
      log.error("Failed to submit project form", err);
      error = "Failed to save project. Please try again.";
    }
  }
</script>

<div class="space-y-6">
  <!-- Success Message -->
  {#if success}
    <Card
      class="mb-6 border-green-200 bg-green-50 dark:border-green-800 dark:bg-green-950"
    >
      <CardContent class="pt-6">
        <div class="flex items-center gap-2 text-green-800 dark:text-green-200">
          <svg
            class="h-5 w-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M5 13l4 4L19 7"
            />
          </svg>
          <span class="font-medium">Success</span>
        </div>
        <p class="mt-1 text-sm text-green-600 dark:text-green-300">{success}</p>
      </CardContent>
    </Card>
  {/if}

  <!-- Error Message -->
  {#if error}
    <Card
      class="mb-6 border-red-200 bg-red-50 dark:border-red-800 dark:bg-red-950"
    >
      <CardContent class="pt-6">
        <div class="flex items-center gap-2 text-red-800 dark:text-red-200">
          <svg
            class="h-5 w-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
          <span class="font-medium">Error</span>
        </div>
        <p class="mt-1 text-sm text-red-600 dark:text-red-300">{error}</p>
      </CardContent>
    </Card>
  {/if}

  <form onsubmit={handleSubmit} class="space-y-8">
    <!-- Basic Information -->
    <Card>
      <CardHeader>
        <div class="flex items-center justify-between">
          <div>
            <CardTitle>Basic Information</CardTitle>
            <CardDescription>
              Essential details about your project
            </CardDescription>
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
                    onclick={handleSync}
                    disabled={isLoading ||
                      isAnalyzing ||
                      !path.trim() ||
                      !isTauri}
                    class="flex items-center gap-2"
                  >
                    <RefreshCw
                      class="h-4 w-4 {isAnalyzing ? 'animate-spin' : ''}"
                    />
                    <span class="hidden sm:inline">Sync</span>
                  </Button>
                {/snippet}
              </TooltipTrigger>
              {#if !isTauri}
                <TooltipContent>
                  <p class="max-w-xs">
                    This feature requires the desktop app. Project analysis and
                    auto-detection need access to the file system. Please use
                    the Tauri desktop application to create or update projects
                    with automatic property detection.
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
            bind:value={path}
            label="Project Path"
            description={isTauri
              ? "Select the directory where your project will be located"
              : "Enter the project path manually. File browser is only available in the desktop app."}
            placeholder="/path/to/your/project"
            disabled={isLoading || isAnalyzing}
            required
            onChange={handlePathChange}
          />
          {#if !isTauri}
            <div
              class="flex items-start gap-2 rounded-md border border-yellow-200 bg-yellow-50 p-3 dark:border-yellow-800 dark:bg-yellow-950"
            >
              <Info
                class="mt-0.5 h-4 w-4 flex-shrink-0 text-yellow-600 dark:text-yellow-400"
              />
              <div class="text-sm text-yellow-800 dark:text-yellow-200">
                <p class="mb-1 font-medium">File browser unavailable</p>
                <p>
                  The folder picker requires the desktop app for file system
                  access. You can still enter the project path manually, but
                  automatic project analysis and property detection won't be
                  available.
                </p>
              </div>
            </div>
          {/if}
        </div>

        <div class="grid grid-cols-1 gap-6 md:grid-cols-2">
          <div class="space-y-2">
            <Label for="name">Project Name *</Label>
            <Input
              id="name"
              bind:value={name}
              placeholder="My Awesome Project"
              disabled={isLoading || isAnalyzing}
              required
            />
          </div>
        </div>

        <div class="space-y-2">
          <Label for="description">Description</Label>
          <Textarea
            id="description"
            bind:value={description}
            placeholder="Brief description of your project..."
            disabled={isLoading || isAnalyzing}
            rows={3}
          />
        </div>
      </CardContent>
    </Card>

    <!-- Framework, Package Manager & Languages -->
    <Card>
      <CardHeader>
        <CardTitle>Framework, Package Manager & Languages</CardTitle>
        <CardDescription>
          Development tools, frameworks, and languages used in your project
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-6">
        <div class="grid grid-cols-1 gap-6 md:grid-cols-3">
          <div class="space-y-2">
            <Label for="frameworks">Frameworks</Label>
            <MultiSelect
              options={frameworks.map((f) => ({
                value: f.id.toString(),
                label: f.name,
                icon: f.icon,
                iconType: f.icon_type,
              }))}
              bind:value={frameworkValueStrings}
              placeholder="Select frameworks..."
              searchPlaceholder="Search frameworks..."
              disabled={isLoading || isAnalyzing || isLoadingEntities}
              class="w-full"
            />
          </div>
          <div class="space-y-2">
            <Label for="package-managers">Package Managers</Label>
            <MultiSelect
              options={packageManagers.map((pm) => ({
                value: pm.id.toString(),
                label: pm.name,
                icon: pm.icon,
                iconType: pm.icon_type,
              }))}
              bind:value={packageManagerValueStrings}
              placeholder="Select package managers..."
              searchPlaceholder="Search package managers..."
              disabled={isLoading || isAnalyzing || isLoadingEntities}
              class="w-full"
            />
          </div>
          <div class="space-y-2">
            <Label for="languages">Languages</Label>
            <MultiSelect
              options={languages.map((lang) => ({
                value: lang.id.toString(),
                label: lang.name,
                icon: lang.icon,
                iconType: lang.icon_type,
              }))}
              bind:value={languageValueStrings}
              placeholder="Select languages..."
              searchPlaceholder="Search languages..."
              disabled={isLoading || isAnalyzing || isLoadingEntities}
              class="w-full"
            />
          </div>
        </div>
      </CardContent>
    </Card>

    <!-- Form Actions -->
    <div class="flex justify-end space-x-4">
      <Button
        type="button"
        variant="outline"
        onclick={onCancel}
        disabled={isLoading || isAnalyzing}
      >
        Cancel
      </Button>
      <Button type="submit" disabled={isLoading || isAnalyzing}>
        {#if isAnalyzing}
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
