<!-- Create Secret Page -->
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
  import { ArrowLeft, Save, Plus, X, Eye, EyeOff } from "@lucide/svelte";
  import { k8sResourceService } from "$lib/domains/cloud/services/k8sResourceService";
  import { toastActions } from "$lib/utils/toast";
  import YamlEditor from "$lib/domains/cloud/components/YamlEditor.svelte";
  import { PageHeader } from "$lib/components/shell";

  let secretName = $state("");
  let namespace = $state($cloudStore.selectedNamespace || "default");
  let secretType = $state("Opaque");
  let yamlMode = $state(false);
  let yamlContent = $state("");
  let dataEntries = $state<
    Array<{ key: string; value: string; visible: boolean }>
  >([{ key: "", value: "", visible: false }]);
  let isLoading = $state(false);

  onMount(async () => {
    // Load namespaces if not already loaded
    if ($cloudStore.resources[ResourceType.NAMESPACE].length === 0) {
      // Namespaces will be loaded by the store
    }
  });

  const namespaceOptions = $derived(() => {
    const namespaces = $cloudStore.resources[ResourceType.NAMESPACE];
    return namespaces.map((ns: any) => ns.name).sort();
  });

  const secretTypes = [
    "Opaque",
    "kubernetes.io/dockerconfigjson",
    "kubernetes.io/tls",
    "kubernetes.io/basic-auth",
  ];

  function addDataEntry() {
    dataEntries = [...dataEntries, { key: "", value: "", visible: false }];
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

  function toggleVisibility(index: number) {
    dataEntries[index].visible = !dataEntries[index].visible;
    dataEntries = [...dataEntries];
  }

  function encodeBase64(value: string): string {
    return btoa(unescape(encodeURIComponent(value)));
  }

  function generateYAML(): string {
    const validEntries = dataEntries.filter(
      (e) => e.key.trim() && e.value.trim(),
    );
    const dataSection = validEntries
      .map((e) => {
        const encoded = encodeBase64(e.value);
        return `  ${e.key}: ${encoded}`;
      })
      .join("\n");

    return `apiVersion: v1
kind: Secret
metadata:
  name: ${secretName}
  namespace: ${namespace}
type: ${secretType}
data:
${dataSection}
`;
  }

  async function handleCreate() {
    if (!secretName.trim()) {
      toastActions.error("Secret name is required");
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
      goto(`/cloud/secrets/${secretName}?namespace=${namespace}`);
    } catch (error) {
      toastActions.error(
        error instanceof Error ? error.message : "Failed to create Secret",
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
    title="Create Secret"
    description="Create a new Kubernetes Secret"
  >
    {#snippet actions()}
      <Button variant="outline" onclick={() => goto("/cloud/secrets")}>
        <ArrowLeft class="mr-2 h-4 w-4" />
        Back to Secrets
      </Button>
    {/snippet}
  </PageHeader>

  <!-- Security Warning -->
  <div
    class="rounded-lg border border-yellow-200 bg-yellow-50 p-4 dark:border-yellow-800 dark:bg-yellow-900/20"
  >
    <p class="text-sm text-yellow-800 dark:text-yellow-200">
      <strong>Security Warning:</strong> Secret values will be base64-encoded. Be
      careful when entering sensitive information.
    </p>
  </div>

  <div class="grid grid-cols-1 gap-6 lg:grid-cols-3">
    <!-- Form Section -->
    <div class="space-y-6 lg:col-span-2">
      {#if yamlMode}
        <!-- YAML Editor -->
        <Card>
          <CardHeader>
            <CardTitle>YAML Editor</CardTitle>
            <p class="mt-1 text-sm text-muted-foreground">
              Edit the Secret YAML directly
            </p>
          </CardHeader>
          <CardContent>
            <YamlEditor value={yamlContent} resourceKind="Secret" {namespace} />
          </CardContent>
        </Card>
      {:else}
        <!-- Form Editor -->
        <Card>
          <CardHeader>
            <CardTitle>Secret Details</CardTitle>
            <p class="mt-1 text-sm text-muted-foreground">
              Configure your Secret
            </p>
          </CardHeader>
          <CardContent class="space-y-4">
            <div class="space-y-2">
              <Label for="name">Name *</Label>
              <Input
                id="name"
                value={secretName}
                oninput={(e) =>
                  (secretName = (e.target as HTMLInputElement).value)}
                placeholder="my-secret"
              />
            </div>

            <div class="space-y-2">
              <Label for="namespace">Namespace *</Label>
              <Select
                bind:value={namespace}
                options={namespaceOptions()}
                placeholder="Select namespace"
              />
            </div>

            <div class="space-y-2">
              <Label for="type">Type</Label>
              <Select bind:value={secretType} options={secretTypes} />
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
                    <div class="relative flex-1">
                      {#if entry.visible}
                        <Textarea
                          value={entry.value}
                          oninput={(e) =>
                            updateDataEntry(
                              index,
                              "value",
                              (e.target as HTMLTextAreaElement).value,
                            )}
                          placeholder="Value"
                          class="min-h-[60px] pr-10"
                        />
                      {:else}
                        <Textarea
                          value={entry.value
                            ? "*".repeat(Math.min(entry.value.length, 20))
                            : ""}
                          oninput={(e) => {
                            const newValue = (e.target as HTMLTextAreaElement)
                              .value;
                            // Only update if not all asterisks
                            if (!newValue.match(/^\*+$/)) {
                              updateDataEntry(index, "value", newValue);
                            }
                          }}
                          placeholder="Value (hidden)"
                          class="min-h-[60px] pr-10"
                        />
                      {/if}
                      <Button
                        variant="ghost"
                        size="sm"
                        class="absolute right-1 top-1 h-8 w-8 p-0"
                        onclick={() => toggleVisibility(index)}
                      >
                        {#if entry.visible}
                          <EyeOff class="h-4 w-4" />
                        {:else}
                          <Eye class="h-4 w-4" />
                        {/if}
                      </Button>
                    </div>
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
          disabled={isLoading || !secretName.trim() || !namespace.trim()}
        >
          <Save class="mr-2 h-4 w-4" />
          {isLoading ? "Creating..." : "Create Secret"}
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
