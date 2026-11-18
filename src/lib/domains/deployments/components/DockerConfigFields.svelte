<!-- Docker-specific deployment form fields -->
<script lang="ts">
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';

  interface Props {
    dockerImageName: string;
    dockerfilePath: string;
    exposedPort: number | undefined;
    projectPath: string;
    portValid: boolean;
  }
  let {
    dockerImageName = $bindable(),
    dockerfilePath = $bindable(),
    exposedPort = $bindable(),
    projectPath,
    portValid,
  }: Props = $props();
</script>

<div class="divider-edge-t divider-edge-full space-y-4 pt-6">
  <h3 class="text-lg font-semibold">Docker Configuration</h3>

  <div>
    <Label for="dockerImageName">Docker Image Name</Label>
    <Input
      id="dockerImageName"
      bind:value={dockerImageName}
      placeholder="my-app:latest (auto-generated if empty)"
    />
    <p class="mt-1 text-xs text-muted-foreground">
      Leave empty to auto-generate based on deployment name
    </p>
  </div>

  <div>
    <Label for="dockerfilePath">Dockerfile Path</Label>
    <Input
      id="dockerfilePath"
      bind:value={dockerfilePath}
      placeholder="{projectPath}/Dockerfile (auto-generated if empty)"
    />
    <p class="mt-1 text-xs text-muted-foreground">Leave empty to auto-generate Dockerfile</p>
  </div>

  <div>
    <Label for="exposedPort">Exposed Port</Label>
    <Input
      id="exposedPort"
      type="number"
      min="1"
      max="65535"
      bind:value={exposedPort}
      placeholder="3000"
      aria-invalid={!portValid}
    />
    {#if !portValid}
      <p class="mt-1 text-xs text-destructive">Port must be between 1 and 65535</p>
    {/if}
  </div>
</div>
