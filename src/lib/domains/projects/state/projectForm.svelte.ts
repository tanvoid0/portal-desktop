/**
 * State and handlers for the create/update project form. Construct it during
 * component init — it registers onMount. Props are read through `getProps` so
 * they stay reactive.
 */
import { onMount } from 'svelte';
import { invokeClient } from '$lib/utils/invokeClient';
import { logger } from '$lib/domains/shared/services/logger';
import { isTauriEnvironment } from '$lib/utils/tauri';
import type { CreateProjectRequest } from '$lib/domains/projects/types';
import { ideService, type Framework } from '$lib/domains/ide';
import { packageManagerService, type PackageManager } from '$lib/domains/package_managers';
import { languageService, type Language } from '$lib/domains/languages';

const log = logger.createScoped('ProjectForm');

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

interface FormProps {
  projectId?: number;
  initialData: Partial<CreateProjectRequest>;
  onSubmit: (data: CreateProjectRequest) => Promise<void>;
}

const toStrings = (ids?: number[]) => (ids || []).map((id) => id.toString());
const toIds = (values: string[]) => values.map((v) => parseInt(v, 10)).filter((id) => !isNaN(id));

/** Ids of the entities whose name matches a detected name, as MultiSelect value strings. */
function matchIds(names: string[], list: { id: number; name: string }[]): string[] {
  const ids = new Set<number>();
  for (const n of names) {
    const found = list.find((e) => e.name.toLowerCase() === n.toLowerCase());
    if (found) ids.add(found.id);
  }
  return [...ids].map((id) => id.toString());
}

export class ProjectFormState {
  // Check if running in Tauri environment (backend-dependent features require this)
  isTauri = isTauriEnvironment();

  name = $state('');
  description = $state('');
  path = $state('');
  frameworkValueStrings = $state<string[]>([]);
  packageManagerValueStrings = $state<string[]>([]);
  languageValueStrings = $state<string[]>([]);

  frameworkIds = $derived(toIds(this.frameworkValueStrings));
  packageManagerIds = $derived(toIds(this.packageManagerValueStrings));
  languageIds = $derived(toIds(this.languageValueStrings));

  // Entity lists
  frameworks = $state<Framework[]>([]);
  packageManagers = $state<PackageManager[]>([]);
  languages = $state<Language[]>([]);
  isLoadingEntities = $state(false);

  error = $state('');
  success = $state('');
  isAnalyzing = $state(false);

  detectedCommands = $state<{
    build_command?: string;
    start_command?: string;
    test_command?: string;
    output_directory?: string;
    dev_port?: number;
    prod_port?: number;
  }>({});

  constructor(private getProps: () => FormProps) {
    const initialData = getProps().initialData;
    this.name = initialData.name || '';
    this.description = initialData.description || '';
    this.path = initialData.path || '';
    this.frameworkValueStrings = toStrings(initialData.framework_ids);
    this.packageManagerValueStrings = toStrings(initialData.package_manager_ids);
    this.languageValueStrings = toStrings(initialData.language_ids);
    this.detectedCommands = {
      build_command: initialData.build_command,
      start_command: initialData.start_command,
      test_command: initialData.test_command,
      output_directory: initialData.output_directory,
      dev_port: initialData.dev_port,
      prod_port: initialData.prod_port,
    };

    // Set up breadcrumbs on mount
    onMount(async () => {
      const projectId = getProps().projectId;
      if (projectId) {
        // This is an update form
        log.info('Initializing project update form', { projectId });
      } else {
        // This is a create form
        log.info('Initializing project create form');
      }
      await this.loadEntities();
    });
  }

  async loadEntities() {
    this.isLoadingEntities = true;
    try {
      await Promise.all([this.loadFrameworks(), this.loadPackageManagers(), this.loadLanguages()]);
    } catch (error) {
      log.warn('Failed to load entities', { error });
    } finally {
      this.isLoadingEntities = false;
    }
  }

  async loadFrameworks() {
    try {
      this.frameworks = await ideService.getAllFrameworks();
    } catch (error) {
      log.warn('Failed to load frameworks', { error });
    }
  }

  async loadPackageManagers() {
    try {
      this.packageManagers = await packageManagerService.getAllPackageManagers();
    } catch (error) {
      log.warn('Failed to load package managers', { error });
    }
  }

  async loadLanguages() {
    try {
      this.languages = await languageService.getAllLanguages();
    } catch (error) {
      log.warn('Failed to load languages', { error });
    }
  }

  handlePathChange = async (newPath: string) => {
    this.path = newPath;

    if (newPath.trim()) {
      await this.analyzeProject(newPath);
    }
  };

  async analyzeProject(projectPath: string, forceSync = false) {
    try {
      this.isAnalyzing = true;
      this.error = '';

      log.info('Analyzing project directory', { path: projectPath, forceSync });

      const analysis: ProjectAnalysis = await invokeClient.post<ProjectAnalysis>(
        'analyze_project_directory',
        {
          path: projectPath,
        }
      );

      log.info('Project analysis completed', analysis);

      // Auto-populate form fields
      if (forceSync || !this.name.trim()) {
        this.name = analysis.name;
      }

      // Always update these fields when syncing; otherwise only fill the empty ones
      if (forceSync || this.frameworkValueStrings.length === 0) {
        if (forceSync || analysis.frameworks.length > 0)
          this.frameworkValueStrings = matchIds(analysis.frameworks, this.frameworks);
      }
      if (forceSync || this.packageManagerValueStrings.length === 0) {
        if (forceSync || analysis.package_managers.length > 0)
          this.packageManagerValueStrings = matchIds(
            analysis.package_managers,
            this.packageManagers
          );
      }
      if (forceSync || this.languageValueStrings.length === 0) {
        if (forceSync || analysis.languages.length > 0)
          this.languageValueStrings = matchIds(analysis.languages, this.languages);
      }

      this.detectedCommands = {
        build_command: analysis.build_command ?? undefined,
        start_command: analysis.start_command ?? undefined,
        test_command: analysis.test_command ?? undefined,
        output_directory: analysis.output_directory ?? undefined,
        dev_port: analysis.dev_port ?? undefined,
        prod_port: analysis.prod_port ?? undefined,
      };

      this.success = forceSync
        ? 'Project properties synced successfully!'
        : 'Project properties auto-detected successfully!';

      // Clear success message after 3 seconds
      setTimeout(() => {
        this.success = '';
      }, 3000);
    } catch (err) {
      log.error('Failed to analyze project', err);
      this.error = 'Failed to analyze project directory. Please check the path and try again.';
    } finally {
      this.isAnalyzing = false;
    }
  }

  handleSync = async () => {
    if (this.path.trim()) {
      await this.analyzeProject(this.path, true);
    } else {
      this.error = 'Please select a project path first.';
    }
  };

  handleSubmit = async (event: SubmitEvent) => {
    event.preventDefault();

    if (!this.name.trim() || !this.path.trim()) {
      this.error = 'Please fill in all required fields.';
      return;
    }

    try {
      this.error = '';
      this.success = '';

      const c = this.detectedCommands;
      const projectData: CreateProjectRequest = {
        name: this.name.trim(),
        description: this.description.trim() || undefined,
        path: this.path.trim(),
        framework_ids: this.frameworkIds,
        package_manager_ids: this.packageManagerIds,
        language_ids: this.languageIds,
        build_command: c.build_command,
        start_command: c.start_command,
        test_command: c.test_command,
        output_directory: c.output_directory,
        dev_port: c.dev_port,
        prod_port: c.prod_port,
      };

      await this.getProps().onSubmit(projectData);
    } catch (err) {
      log.error('Failed to submit project form', err);
      this.error =
        err instanceof Error && err.message
          ? err.message
          : 'Failed to save project. Please try again.';
    }
  };
}
