<!-- CLI-specific deployment form fields -->
<script lang="ts">
  import { Input } from '$lib/components/ui/input';
  import { Label } from '$lib/components/ui/label';

  interface Props {
    command: string;
    workingDirectory: string;
    projectPath: string;
  }
  let { command = $bindable(), workingDirectory = $bindable(), projectPath }: Props = $props();
</script>

<div class="divider-edge-t divider-edge-full space-y-4 pt-6">
  <h3 class="text-lg font-semibold">CLI Configuration</h3>

  <div>
    <Label for="command">Command *</Label>
    <Input id="command" bind:value={command} placeholder="npm start" required />
    <p class="mt-1 text-xs text-muted-foreground">
      Enter the command to run (e.g., "npm start", "python app.py", "cargo run")
    </p>
  </div>

  <div>
    <Label for="workingDirectory">Working Directory</Label>
    <Input
      id="workingDirectory"
      bind:value={workingDirectory}
      placeholder="{projectPath} (defaults to project path)"
    />
  </div>
</div>
