<!--
	Create New Deployment Page
-->

<script lang="ts">
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';
  import { Textarea } from '$lib/components/ui/textarea';
  import Select from '$lib/components/ui/select.svelte';
  import DockerConfigFields from '$lib/domains/deployments/components/DockerConfigFields.svelte';
  import CliConfigFields from '$lib/domains/deployments/components/CliConfigFields.svelte';
  import EnvVarsEditor from '$lib/domains/deployments/components/EnvVarsEditor.svelte';
  import {
    DeploymentType,
    ProjectType,
    type DeploymentCreateRequest,
    type EnvFile,
  } from '$lib/domains/deployments/types';
  import { deploymentActions } from '$lib/domains/deployments/stores/deploymentStore';
  import { deploymentService } from '$lib/domains/deployments/services/deploymentService';
  import { loadDeployConfig, saveDeployConfig } from '$lib/domains/deployments/utils/deployConfig';
  import { toast } from '$lib/utils/toast';
  import { logger } from '$lib/domains/shared';
  import Loader2 from '~icons/fa6-solid/spinner';
  import { PageHeader } from '$lib/components/shell';

  let deploymentType = $state<DeploymentType>(DeploymentType.DOCKER);
  let name = $state('');
  let description = $state('');
  let projectPath = $state('');
  let projectType = $state<ProjectType>(ProjectType.NODE);
  let sdkVersion = $state('latest');

  // Docker-specific fields
  let dockerImageName = $state('');
  let dockerfilePath = $state('');
  let exposedPort = $state<number | undefined>(3000);

  // CLI-specific fields
  let command = $state('');
  let workingDirectory = $state('');

  // Environment variables
  let envVars = $state<Record<string, string>>({});
  let isSubmitting = $state(false);

  // Project .env* files available to seed env vars, and which one is loaded.
  let envFiles = $state<EnvFile[]>([]);
  let selectedEnvFile = $state<string>('');

  // Load env vars from a project .env* file, replacing the current list. "None"
  // (empty) just clears the selection and leaves any manual edits in place.
  function applyEnvFile(fileName: string) {
    selectedEnvFile = fileName;
    const file = envFiles.find((f) => f.name === fileName);
    if (!file) return;
    const next: Record<string, string> = {};
    for (const { key, value } of file.vars) next[key] = value;
    envVars = next;
  }

  // Prefill when opened from a project's Deploy button (?projectPath=&name=),
  // restore the project's remembered config, then offer its .env files.
  onMount(async () => {
    const params = new URLSearchParams(window.location.search);
    const p = params.get('projectPath');
    const n = params.get('name');
    if (p) projectPath = p;
    if (n) name = n;

    const saved = projectPath ? loadDeployConfig(projectPath) : null;
    if (saved) {
      deploymentType = saved.deploymentType ?? deploymentType;
      projectType = saved.projectType ?? projectType;
      description = saved.description ?? description;
      exposedPort = saved.exposedPort ?? exposedPort;
      dockerImageName = saved.dockerImageName ?? dockerImageName;
      dockerfilePath = saved.dockerfilePath ?? dockerfilePath;
      command = saved.command ?? command;
      workingDirectory = saved.workingDirectory ?? workingDirectory;
      envVars = saved.envVars ?? envVars;
      selectedEnvFile = saved.selectedEnvFile ?? '';
    }

    if (projectPath) {
      envFiles = await deploymentService.listProjectEnvFiles(projectPath);
      // First-time setup with nothing entered yet: default to .env.local.
      if (!saved && Object.keys(envVars).length === 0) {
        const preferred = envFiles.find((f) => f.name === '.env.local') ?? envFiles[0];
        if (preferred) applyEnvFile(preferred.name);
      }
    }
  });

  const portValid = $derived(
    deploymentType !== DeploymentType.DOCKER ||
      exposedPort == null ||
      (Number.isInteger(exposedPort) && exposedPort >= 1 && exposedPort <= 65535)
  );

  const canSubmit = $derived(
    !!name.trim() &&
      !!projectPath.trim() &&
      portValid &&
      !(deploymentType === DeploymentType.CLI && !command.trim()) &&
      !isSubmitting
  );

  async function handleSubmit() {
    if (!name.trim() || !projectPath.trim()) {
      toast.error('Name and project path are required');
      return;
    }

    if (!portValid) {
      toast.error('Port must be between 1 and 65535');
      return;
    }

    // Validate type-specific fields
    if (deploymentType === DeploymentType.CLI && !command.trim()) {
      toast.error('Command is required for CLI deployments');
      return;
    }

    isSubmitting = true;

    try {
      const request: DeploymentCreateRequest = {
        name: name.trim(),
        description: description || undefined,
        type: deploymentType,
        projectPath: projectPath.trim(),
        projectType,
        environment: {
          variables: envVars,
          secrets: [],
          volumes: [],
          ports: [],
          networks: [],
        },
        // Docker-specific
        dockerImageName:
          deploymentType === DeploymentType.DOCKER ? dockerImageName || undefined : undefined,
        dockerfilePath:
          deploymentType === DeploymentType.DOCKER ? dockerfilePath || undefined : undefined,
        exposedPort: deploymentType === DeploymentType.DOCKER ? exposedPort : undefined,
        // CLI-specific
        command: deploymentType === DeploymentType.CLI ? command.trim() : undefined,
        workingDirectory:
          deploymentType === DeploymentType.CLI ? workingDirectory || projectPath : undefined,
      };

      // Remember this project's choices so its Deploy button is one-click next time.
      saveDeployConfig(projectPath, {
        deploymentType,
        projectType,
        description,
        exposedPort,
        dockerImageName,
        dockerfilePath,
        command,
        workingDirectory,
        envVars,
        selectedEnvFile: selectedEnvFile || undefined,
      });

      // Create builds the image; start runs the container — one action.
      await deploymentActions.deployAndStart(request);
      toast.success('Deployment built and started');
      goto('/deployments');
    } catch (error) {
      logger.error('Failed to create deployment', {
        context: 'CreateDeploymentPage',
        error,
      });
      // Backend errors come back as plain strings (the real docker/build output),
      // not Error instances — surface them instead of a generic message.
      const raw =
        error instanceof Error
          ? error.message
          : typeof error === 'string'
            ? error
            : 'Failed to create deployment';
      toast.error(raw.length > 300 ? `${raw.slice(0, 300)}…` : raw);
    } finally {
      isSubmitting = false;
    }
  }
</script>

<svelte:head>
  <title>Create Deployment - Portal Desktop</title>
</svelte:head>

<div class="mx-auto max-w-4xl space-y-6">
  <PageHeader title="New deployment">
    {#snippet actions()}
      <Button
        variant="outline"
        size="sm"
        onclick={() => goto('/deployments')}
        disabled={isSubmitting}
      >
        Cancel
      </Button>
      <Button size="sm" type="submit" form="deployment-form" disabled={!canSubmit}>
        {#if isSubmitting}
          <Loader2 class="mr-2 h-4 w-4 animate-spin" />
          Building &amp; starting…
        {:else}
          Deploy (build &amp; run)
        {/if}
      </Button>
    {/snippet}
  </PageHeader>

  <Card>
    <CardHeader>
      <CardTitle>Deployment Configuration</CardTitle>
    </CardHeader>
    <CardContent>
      <form
        id="deployment-form"
        onsubmit={(e) => {
          e.preventDefault();
          handleSubmit();
        }}
        class="space-y-6"
      >
        <!-- Deployment Type -->
        <div>
          <Label for="deploymentType">Deployment Type *</Label>
          <Select
            options={[
              { value: DeploymentType.DOCKER, label: 'Docker' },
              { value: DeploymentType.CLI, label: 'CLI Command' },
            ]}
            defaultValue={deploymentType}
            onSelect={(value) => {
              deploymentType = value as DeploymentType;
            }}
          />
        </div>

        <!-- Basic Fields -->
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
          <div>
            <Label for="name">Name *</Label>
            <Input id="name" bind:value={name} placeholder="My Deployment" required />
          </div>

          <div>
            <Label for="projectType">Project Type *</Label>
            <Select
              options={Object.values(ProjectType)}
              defaultValue={projectType}
              onSelect={(value) => {
                projectType = value as ProjectType;
              }}
            />
          </div>
        </div>

        <div>
          <Label for="description">Description</Label>
          <Textarea
            id="description"
            bind:value={description}
            placeholder="Optional description"
            rows={2}
          />
        </div>

        <div>
          <Label for="projectPath">Project Path *</Label>
          <Input
            id="projectPath"
            bind:value={projectPath}
            placeholder="/path/to/project"
            required
          />
        </div>

        <div>
          <Label for="sdkVersion">SDK Version</Label>
          <Input id="sdkVersion" bind:value={sdkVersion} placeholder="latest" />
        </div>

        {#if deploymentType === DeploymentType.DOCKER}
          <DockerConfigFields
            bind:dockerImageName
            bind:dockerfilePath
            bind:exposedPort
            {projectPath}
            {portValid}
          />
        {/if}

        {#if deploymentType === DeploymentType.CLI}
          <CliConfigFields bind:command bind:workingDirectory {projectPath} />
        {/if}

        <EnvVarsEditor bind:envVars {envFiles} {selectedEnvFile} onSelectFile={applyEnvFile} />
      </form>
    </CardContent>
  </Card>
</div>
