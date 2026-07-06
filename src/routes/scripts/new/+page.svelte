<!--
	Create New Script Page
	Dedicated page for creating new scripts with JSON import support
-->
<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Input } from "$lib/components/ui/input";
  import { Textarea } from "$lib/components/ui/textarea";
  import { Badge } from "$lib/components/ui/badge";
  import { Label } from "$lib/components/ui/label";
  import Select from "$lib/components/ui/select.svelte";
  import {
    ArrowLeft,
    Upload,
    Trash2,
    Plus,
    ChevronDown,
    ChevronUp,
    FileCode,
    Save,
  } from "@lucide/svelte";
  import type {
    BlockParameter,
    CreateBlockRequest,
  } from "$lib/domains/projects/pipelines";
  import { blockLibraryService } from "$lib/domains/projects/pipelines";
  import { toast } from "$lib/utils/toast";
  import { setBreadcrumbs } from "$lib/domains/shared/stores/breadcrumbStore";

  // Set breadcrumb
  setBreadcrumbs([
    { label: "Scripts", href: "/scripts" },
    { label: "New Script", href: "/scripts/new" },
  ]);

  // Check if we should auto-open import section from URL query
  onMount(() => {
    const importParam = $page.url.searchParams.get("import");
    if (importParam === "true") {
      showJsonImport = true;
    }
  });

  // Form state
  let formData = $state<CreateBlockRequest>({
    name: "",
    description: "",
    category: "utility",
    parameters: [],
    command: "",
    executionType: "script",
    defaultConfig: {},
    tags: [],
  });

  // JSON import state
  let jsonImportText = $state("");
  let jsonImportError = $state("");
  let showJsonImport = $state(false);

  const jsonImportPlaceholder =
    '{"name": "My Script", "command": "./script.sh", "parameters": [], ...}';
  const commandPlaceholder =
    "e.g., ./scripts/vpn.sh ${action} --config-dir ${configDir}";

  // Parameter editor state
  let showAddParameter = $state(false);
  let newParameter = $state<BlockParameter>({
    name: "",
    type: "string",
    description: "",
    required: false,
    defaultValue: "",
  });

  let saving = $state(false);

  function handleJsonImport() {
    jsonImportError = "";
    if (!jsonImportText.trim()) {
      jsonImportError = "Please paste JSON content";
      return;
    }
    try {
      const parsed = JSON.parse(jsonImportText);
      formData = {
        name: parsed.name || "",
        description: parsed.description || "",
        category: parsed.category || "utility",
        parameters: parsed.parameters || [],
        command: parsed.command || "",
        executionType: parsed.executionType || "script",
        defaultConfig: parsed.defaultConfig || {},
        tags: parsed.tags || [],
      };
      jsonImportText = "";
      showJsonImport = false;
      toast.success("JSON imported - review and save");
    } catch (e) {
      jsonImportError = "Invalid JSON format";
    }
  }

  function handleJsonFileImport(event: Event) {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (e) => {
      jsonImportText = e.target?.result as string;
      handleJsonImport();
    };
    reader.readAsText(file);
    target.value = "";
  }

  async function handleSave() {
    if (!formData.name || !formData.description || !formData.command) {
      toast.error("Please fill in all required fields");
      return;
    }

    saving = true;
    try {
      // Build defaultConfig from parameters with defaultValue
      const defaultConfig: Record<string, any> = {};
      formData.parameters.forEach((p) => {
        if (p.defaultValue !== undefined && p.defaultValue !== "") {
          defaultConfig[p.name] = p.defaultValue;
        }
      });
      formData.defaultConfig = defaultConfig;

      await blockLibraryService.createBlock(formData);
      toast.success("Script created successfully");
      goto("/scripts");
    } catch (error) {
      console.error("Failed to create script", error);
      toast.error("Failed to create script");
    } finally {
      saving = false;
    }
  }

  function addParameter() {
    if (!newParameter.name.trim()) {
      toast.error("Parameter name is required");
      return;
    }
    if (formData.parameters.some((p) => p.name === newParameter.name)) {
      toast.error("Parameter name already exists");
      return;
    }
    formData.parameters = [...formData.parameters, { ...newParameter }];
    newParameter = {
      name: "",
      type: "string",
      description: "",
      required: false,
      defaultValue: "",
    };
    showAddParameter = false;
  }

  function removeParameter(index: number) {
    formData.parameters = formData.parameters.filter((_, i) => i !== index);
  }

  const isValid = $derived(
    formData.name && formData.description && formData.command,
  );
</script>

<svelte:head>
  <title>New Script - Portal Desktop</title>
</svelte:head>

<div class="container mx-auto max-w-4xl p-6">
  <!-- Header -->
  <div class="mb-6 flex items-center gap-4">
    <Button variant="ghost" size="icon" onclick={() => goto("/scripts")}>
      <ArrowLeft class="h-5 w-5" />
    </Button>
    <div class="flex-1">
      <h1 class="flex items-center gap-2 text-2xl font-bold">
        <FileCode class="h-6 w-6" />
        Create New Script
      </h1>
      <p class="text-muted-foreground">
        Define a reusable script with configurable parameters
      </p>
    </div>
    <Button onclick={handleSave} disabled={!isValid || saving}>
      <Save class="mr-2 h-4 w-4" />
      {saving ? "Saving..." : "Save Script"}
    </Button>
  </div>

  <div class="space-y-6">
    <!-- JSON Import Card -->
    <Card>
      <CardHeader class="pb-3">
        <button
          class="flex w-full items-center justify-between text-left"
          onclick={() => (showJsonImport = !showJsonImport)}
        >
          <div>
            <CardTitle class="text-base">Import from JSON</CardTitle>
            <CardDescription
              >Quickly populate fields from a JSON file or paste</CardDescription
            >
          </div>
          {#if showJsonImport}
            <ChevronUp class="h-5 w-5 text-muted-foreground" />
          {:else}
            <ChevronDown class="h-5 w-5 text-muted-foreground" />
          {/if}
        </button>
      </CardHeader>
      {#if showJsonImport}
        <CardContent class="space-y-4">
          <div class="flex gap-2">
            <input
              type="file"
              id="json-file-import"
              accept=".json"
              class="hidden"
              onchange={handleJsonFileImport}
            />
            <Button
              variant="outline"
              onclick={() =>
                document.getElementById("json-file-import")?.click()}
            >
              <Upload class="mr-2 h-4 w-4" />
              Load from File
            </Button>
          </div>
          <div>
            <Label>Or paste JSON directly</Label>
            <Textarea
              bind:value={jsonImportText}
              placeholder={jsonImportPlaceholder}
              rows={6}
              class="mt-2 font-mono text-sm"
            />
          </div>
          {#if jsonImportError}
            <p class="text-sm text-destructive">{jsonImportError}</p>
          {/if}
          <Button onclick={handleJsonImport} disabled={!jsonImportText.trim()}>
            Import & Preview
          </Button>
        </CardContent>
      {/if}
    </Card>

    <!-- Basic Information -->
    <Card>
      <CardHeader>
        <CardTitle>Basic Information</CardTitle>
        <CardDescription
          >Script name, description, and categorization</CardDescription
        >
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="grid grid-cols-2 gap-4">
          <div>
            <Label for="name">Name *</Label>
            <Input
              id="name"
              bind:value={formData.name}
              placeholder="e.g., VPN Connect"
              class="mt-1"
            />
          </div>
          <div>
            <Label for="category">Category *</Label>
            <Select
              options={[
                { value: "utility", label: "Utility" },
                { value: "build", label: "Build" },
                { value: "test", label: "Test" },
                { value: "deploy", label: "Deploy" },
                { value: "custom", label: "Custom" },
              ]}
              bind:value={formData.category}
              placeholder="Select category"
              class="mt-1"
            />
          </div>
        </div>
        <div>
          <Label for="description">Description *</Label>
          <Textarea
            id="description"
            bind:value={formData.description}
            placeholder="Describe what this script does..."
            rows={3}
            class="mt-1"
          />
        </div>
        <div>
          <Label for="tags">Tags</Label>
          <Input
            id="tags"
            value={(formData.tags || []).join(", ")}
            oninput={(e) => {
              formData.tags = (e.target as HTMLInputElement).value
                .split(",")
                .map((t) => t.trim())
                .filter((t) => t.length > 0);
            }}
            placeholder="e.g., vpn, network, security (comma-separated)"
            class="mt-1"
          />
        </div>
      </CardContent>
    </Card>

    <!-- Command Configuration -->
    <Card>
      <CardHeader>
        <CardTitle>Command Configuration</CardTitle>
        <CardDescription
          >Define the command or script to execute</CardDescription
        >
      </CardHeader>
      <CardContent class="space-y-4">
        <div>
          <Label for="command">Command / Script Path *</Label>
          <Textarea
            id="command"
            bind:value={formData.command}
            placeholder={commandPlaceholder}
            rows={3}
            class="mt-1 font-mono text-sm"
          />
          <p class="mt-2 text-xs text-muted-foreground">
            Use <code class="rounded bg-muted px-1">${"{paramName}"}</code> for parameter
            placeholders. These will be replaced with actual values when the script
            runs.
          </p>
        </div>
        <div>
          <Label for="executionType">Execution Type</Label>
          <Select
            options={[
              { value: "script", label: "Script" },
              { value: "command", label: "Command" },
              { value: "docker", label: "Docker" },
            ]}
            bind:value={formData.executionType}
            placeholder="Select type"
            class="mt-1"
          />
        </div>
      </CardContent>
    </Card>

    <!-- Parameters -->
    <Card>
      <CardHeader>
        <div class="flex items-center justify-between">
          <div>
            <CardTitle>Parameters</CardTitle>
            <CardDescription
              >Define configurable parameters for your script</CardDescription
            >
          </div>
          <Button
            variant="outline"
            onclick={() => (showAddParameter = !showAddParameter)}
          >
            <Plus class="mr-2 h-4 w-4" />
            Add Parameter
          </Button>
        </div>
      </CardHeader>
      <CardContent class="space-y-4">
        {#if showAddParameter}
          <div class="space-y-4 rounded-lg border bg-muted/30 p-4">
            <h4 class="font-medium">New Parameter</h4>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <Label class="text-sm">Name *</Label>
                <Input
                  bind:value={newParameter.name}
                  placeholder="e.g., configDir"
                  class="mt-1"
                />
              </div>
              <div>
                <Label class="text-sm">Type</Label>
                <Select
                  options={[
                    { value: "string", label: "String" },
                    { value: "number", label: "Number" },
                    { value: "boolean", label: "Boolean" },
                    { value: "select", label: "Select" },
                    { value: "file", label: "File" },
                    { value: "directory", label: "Directory" },
                  ]}
                  bind:value={newParameter.type}
                  class="mt-1"
                />
              </div>
            </div>
            <div>
              <Label class="text-sm">Description</Label>
              <Input
                bind:value={newParameter.description}
                placeholder="Describe this parameter..."
                class="mt-1"
              />
            </div>
            <div class="grid grid-cols-2 gap-4">
              <div>
                <Label class="text-sm">Default Value</Label>
                <Input
                  bind:value={newParameter.defaultValue}
                  placeholder="Default value (optional)"
                  class="mt-1"
                />
              </div>
              <div class="flex items-end pb-2">
                <label class="flex items-center gap-2">
                  <input
                    type="checkbox"
                    bind:checked={newParameter.required}
                    class="rounded"
                  />
                  <span class="text-sm">Required parameter</span>
                </label>
              </div>
            </div>
            <div class="flex gap-2">
              <Button onclick={addParameter}>Add Parameter</Button>
              <Button
                variant="outline"
                onclick={() => (showAddParameter = false)}>Cancel</Button
              >
            </div>
          </div>
        {/if}

        {#if formData.parameters.length > 0}
          <div class="divide-y rounded-lg border">
            {#each formData.parameters as param, i}
              <div class="flex items-center justify-between p-4">
                <div class="space-y-1">
                  <div class="flex items-center gap-2">
                    <code
                      class="rounded bg-muted px-2 py-0.5 font-mono text-sm font-medium"
                    >
                      ${"{" + param.name + "}"}
                    </code>
                    <Badge variant="outline">{param.type}</Badge>
                    {#if param.required}
                      <Badge variant="destructive" class="text-xs"
                        >required</Badge
                      >
                    {/if}
                  </div>
                  {#if param.description}
                    <p class="text-sm text-muted-foreground">
                      {param.description}
                    </p>
                  {/if}
                  {#if param.defaultValue !== undefined && param.defaultValue !== ""}
                    <p class="text-xs text-muted-foreground">
                      Default: <code class="rounded bg-muted px-1"
                        >{param.defaultValue}</code
                      >
                    </p>
                  {/if}
                </div>
                <Button
                  variant="ghost"
                  size="icon"
                  onclick={() => removeParameter(i)}
                >
                  <Trash2 class="h-4 w-4" />
                </Button>
              </div>
            {/each}
          </div>
        {:else}
          <div class="py-8 text-center text-muted-foreground">
            <p>No parameters defined yet.</p>
            <p class="text-sm">
              Parameters allow users to customize script behavior at runtime.
            </p>
          </div>
        {/if}
      </CardContent>
    </Card>

    <!-- Preview -->
    {#if formData.name || formData.command}
      <Card>
        <CardHeader>
          <CardTitle>Preview</CardTitle>
          <CardDescription>JSON representation of your script</CardDescription>
        </CardHeader>
        <CardContent>
          <pre
            class="max-h-60 overflow-x-auto rounded-lg bg-muted p-4 font-mono text-xs">{JSON.stringify(
              formData,
              null,
              2,
            )}</pre>
        </CardContent>
      </Card>
    {/if}

    <!-- Actions -->
    <div class="flex justify-end gap-2 pb-6">
      <Button variant="outline" onclick={() => goto("/scripts")}>Cancel</Button>
      <Button onclick={handleSave} disabled={!isValid || saving}>
        <Save class="mr-2 h-4 w-4" />
        {saving ? "Saving..." : "Save Script"}
      </Button>
    </div>
  </div>
</div>
