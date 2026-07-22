<!-- Create ConfigMap Page -->
<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { cloudStore } from "$lib/domains/cloud/stores";
  import { ResourceType } from "$lib/domains/cloud/core/types";
  import {
    Card,
    CardContent,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Label } from "$lib/components/ui/label";
  import { Textarea } from "$lib/components/ui/textarea";
  import Select from "$lib/components/ui/select.svelte";
  import { ArrowLeft, Save, Plus, X } from "@lucide/svelte";
  import { k8sResourceService } from "$lib/domains/cloud/services/k8sResourceService";
  import { toastActions } from "$lib/utils/toast";
  import YamlEditor from "$lib/domains/cloud/components/YamlEditor.svelte";
  import { PageHeader } from "$lib/components/shell";

  let configMapName = $state("");
  let namespace = $state($cloudStore.selectedNamespace || "default");
  let yamlMode = $state(false);
  let yamlContent = $state("");
  let dataEntries = $state<Array<{ key: string; value: string }>>([
    { key: "", value: "" },
  ]);
  let isLoading = $state(false);

  onMount(async () => {
    // Load namespaces if not already loaded
    if ($cloudStore.resources[ResourceType.NAMESPACE].length === 0) {
      // Namespaces will be loaded by the store
    }
  });

  const namespaceOptions = $derived.by(() => {
    const namespaces = $cloudStore.resources[ResourceType.NAMESPACE];
    return namespaces.map((ns: any) => ns.name).sort();
  });

  function addDataEntry() {
    dataEntries = [...dataEntries, { key: "", value: "" }];
  }

  function removeDataEntry(index: number) {
    dataEntries = dataEntries.filter((_, i) => i !== index);
  }

  function updateDataEntry(
    index: number,
    field: "key" | "value",
    value: string,
  ) {
    dataEntries[index] = { ...dataEntries[index], [field]: value };
    dataEntries = [...dataEntries];
  }

  function generateYAML(): string {
    const validEntries = dataEntries.filter(
      (e) => e.key.trim() && e.value.trim(),
    );
    const dataSection = validEntries
      .map((e) => `  ${e.key}: ${e.value}`)
      .join("\n");

    return `apiVersion: v1
kind: ConfigMap
metadata:
  name: ${configMapName}
  namespace: ${namespace}
data:
${dataSection}
`;
  }

  async function handleCreate() {
    if (!configMapName.trim()) {
      toastActions.error("ConfigMap name is required");
      return;
    }

    if (!namespace.trim()) {
      toastActions.error("Namespace is required");
      return;
    }

    isLoading = true;

    try {
      let yaml = yamlContent;

      if (!yamlMode) {
        // Generate YAML from form data
        yaml = generateYAML();
      }

      const result = await k8sResourceService.applyResourceYaml(namespace, yaml);

      toastActions.success(result);
      goto(`/cloud/configmaps/${configMapName}?namespace=${namespace}`);
    } catch (error) {
      toastActions.error(
        error instanceof Error ? error.message : "Failed to create ConfigMap",
      );
    } finally {
      isLoading = false;
    }
  }

  function toggleMode() {
    if (yamlMode) {
      // Switching from YAML to form - parse YAML if possible
      // For now, just clear and let user use form
      yamlContent = "";
    } else {
      // Switching from form to YAML - generate YAML
      yamlContent = generateYAML();
    }
    yamlMode = !yamlMode;
  }

  const validEntries = $derived(
    dataEntries.filter((e) => e.key.trim() && e.value.trim()),
  );
</script>

<div class="space-y-6">
  <PageHeader
    title="Create ConfigMap"
    description="Create a new Kubernetes ConfigMap"
  >
    {#snippet actions()}
      <Button variant="outline" onclick={() => goto("/cloud/configmaps")}>
        <ArrowLeft class="mr-2 h-4 w-4" />
        Back to ConfigMaps
      </Button>
    {/snippet}
  </PageHeader>

  <div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
    <!-- Form Section -->
    <div class="space-y-6 lg:col-span-2">
      {#if yamlMode}
        <!-- YAML Editor -->
        <Card>
          <CardHeader>
            <CardTitle>YAML Editor</CardTitle>
            <p class="mt-1 text-sm text-muted-foreground">
              Edit the ConfigMap YAML directly
            </p>
          </CardHeader>
          <CardContent>
            <YamlEditor
              value={yamlContent}
              resourceKind="ConfigMap"
              {namespace}
            />
          </CardContent>
        </Card>
      {:else}
        <!-- Form Editor -->
        <Card>
          <CardHeader>
            <CardTitle>ConfigMap Details</CardTitle>
            <p class="mt-1 text-sm text-muted-foreground">
              Configure your ConfigMap
            </p>
          </CardHeader>
          <CardContent class="space-y-4">
            <div class="space-y-2">
              <Label for="name">Name *</Label>
              <Input
                id="name"
                value={configMapName}
                oninput={(e) =>
                  (configMapName = (e.target as HTMLInputElement).value)}
                placeholder="my-configmap"
              />
            </div>

            <div class="space-y-2">
              <Label for="namespace">Namespace *</Label>
              <Select
                bind:value={namespace}
                options={namespaceOptions}
                placeholder="Select namespace"
              />
            </div>

            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <Label>Data</Label>
                <Button variant="outline" size="sm" onclick={addDataEntry}>
                  <Plus class="mr-2 h-4 w-4" />
                  Add Entry
                </Button>
              </div>

              <div class="space-y-2">
                {#each dataEntries as entry, index}
                  <div class="flex gap-2">
                    <Input
                      value={entry.key}
                      oninput={(e) =>
                        updateDataEntry(
                          index,
                          "key",
                          (e.target as HTMLInputElement).value,
                        )}
                      placeholder="Key"
                      class="flex-1"
                    />
                    <Textarea
                      value={entry.value}
                      oninput={(e) =>
                        updateDataEntry(
                          index,
                          "value",
                          (e.target as HTMLTextAreaElement).value,
                        )}
                      placeholder="Value"
                      class="min-h-[60px] flex-1"
                    />
                    <Button
                      variant="ghost"
                      size="sm"
                      onclick={() => removeDataEntry(index)}
                      disabled={dataEntries.length === 1}
                    >
                      <X class="h-4 w-4" />
                    </Button>
                  </div>
                {/each}
              </div>
            </div>
          </CardContent>
        </Card>
      {/if}

      <div class="flex items-center justify-end gap-2">
        <Button variant="outline" onclick={toggleMode}>
          {yamlMode ? "Switch to Form" : "Switch to YAML"}
        </Button>
        <Button
          onclick={handleCreate}
          disabled={isLoading || !configMapName.trim() || !namespace.trim()}
        >
          <Save class="mr-2 h-4 w-4" />
          {isLoading ? "Creating..." : "Create ConfigMap"}
        </Button>
      </div>
    </div>

    <!-- Preview Section -->
    <div class="lg:col-span-1">
      <Card>
        <CardHeader>
          <CardTitle>Preview</CardTitle>
        </CardHeader>
        <CardContent>
          <pre
            class="overflow-auto rounded bg-muted p-4 text-xs">{generateYAML()}</pre>
        </CardContent>
      </Card>
    </div>
  </div>
</div>
