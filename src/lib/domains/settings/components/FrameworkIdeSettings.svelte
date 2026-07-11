<!--
	Framework IDE Settings - Map frameworks to preferred IDEs
-->

<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { onMount } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Label } from "$lib/components/ui/label";
  import { Badge } from "$lib/components/ui/badge";
  import Select from "$lib/components/ui/select.svelte";
  import {
    Link2,
    Plus,
    Edit,
    Trash2,
    ArrowRight,
    RefreshCw,
    Loader2,
  } from "@lucide/svelte";
  import { toast } from "$lib/utils/toast";
  import { confirmAction } from "$lib/utils/confirm";
  import { logger } from "$lib/domains/shared";
  import {
    ideService,
    type IdeConfig,
    type FrameworkIdeMapping,
  } from "$lib/domains/ide";

  let mappings = $state<FrameworkIdeMapping[]>([]);
  let ides = $state<IdeConfig[]>([]);
  let isLoadingMappings = $state(false);
  let isLoadingIdes = $state(false);
  let showModal = $state(false);
  let editingMapping = $state<FrameworkIdeMapping | null>(null);
  let mappingFramework = $state("");
  let mappingIdeId = $state<string>("");

  onMount(async () => {
    await Promise.all([loadMappings(), loadIdes()]);
  });

  async function loadMappings() {
    try {
      isLoadingMappings = true;
      mappings = await ideService.getAllFrameworkIdeMappings();
      logger.info("Framework IDE mappings loaded", {
        context: "FrameworkIdeSettings",
        count: mappings.length,
      });
    } catch (error: any) {
      toast.error("Failed to load framework IDE mappings", error);
    } finally {
      isLoadingMappings = false;
    }
  }

  async function loadIdes() {
    try {
      isLoadingIdes = true;
      ides = await ideService.getAllIdes();
    } catch (error: any) {
      toast.error("Failed to load IDEs", error);
    } finally {
      isLoadingIdes = false;
    }
  }

  function getIdeName(ideId: number): string {
    const ide = ides.find((i) => i.id === ideId);
    return ide?.name || `IDE ${ideId}`;
  }

  function startAdding() {
    editingMapping = null;
    mappingFramework = "";
    mappingIdeId = "";
    showModal = true;
  }

  function startEditing(mapping: FrameworkIdeMapping) {
    editingMapping = mapping;
    mappingFramework = mapping.framework;
    mappingIdeId = String(mapping.ide_id);
    showModal = true;
  }

  function closeModal() {
    showModal = false;
    editingMapping = null;
    mappingFramework = "";
    mappingIdeId = "";
  }

  function handleModalKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      closeModal();
    }
  }

  async function saveMapping() {
    if (!mappingFramework.trim() || !mappingIdeId) {
      toast.error("Framework name and IDE selection are required");
      return;
    }

    try {
      const ideId = parseInt(mappingIdeId);
      await ideService.setFrameworkIdeMapping(mappingFramework.trim(), ideId);
      toast.success(
        editingMapping
          ? "Framework IDE mapping updated successfully"
          : "Framework IDE mapping created successfully",
      );
      closeModal();
      await loadMappings();
    } catch (error: any) {
      toast.error("Failed to save framework IDE mapping", error);
    }
  }

  async function deleteMapping(mapping: FrameworkIdeMapping) {
    const confirmed = await confirmAction(
      `Are you sure you want to delete the mapping for "${mapping.framework}"?`,
      "Delete mapping",
    );
    if (!confirmed) return;

    try {
      await ideService.deleteFrameworkIdeMapping(mapping.framework);
      toast.success("Framework IDE mapping deleted successfully");
      await loadMappings();
    } catch (error: any) {
      toast.error("Failed to delete framework IDE mapping", error);
    }
  }
</script>

<div class="space-y-6">
  <!-- Header -->
  <div class="flex items-center justify-between">
    <div>
      <h2 class="flex items-center gap-2 text-2xl font-bold tracking-tight">
        <Link2 class="h-6 w-6" />
        Framework IDE Mappings
      </h2>
      <p class="text-muted-foreground">
        Map frameworks to their preferred IDEs for automatic project opening
      </p>
    </div>
    <div class="flex gap-2">
      <Button
        variant="outline"
        size="sm"
        onclick={loadMappings}
        disabled={isLoadingMappings}
      >
        <RefreshCw class="mr-2 h-4 w-4" />
        Refresh
      </Button>
      <Button onclick={startAdding}>
        <Plus class="mr-2 h-4 w-4" />
        Add Mapping
      </Button>
    </div>
  </div>

  <!-- Mappings List -->
  <Card>
    <CardHeader>
      <CardTitle class="flex items-center gap-2">
        <Link2 class="h-5 w-5" />
        Framework IDE Mappings
        <Badge variant="outline">{mappings.length}</Badge>
      </CardTitle>
      <CardDescription>
        Configure which IDE should open for each framework
      </CardDescription>
    </CardHeader>
    <CardContent>
      {#if isLoadingMappings}
        <div class="flex items-center justify-center py-8">
          <Loader2 class="h-6 w-6 animate-spin text-muted-foreground" />
          <span class="ml-2 text-muted-foreground">Loading mappings...</span>
        </div>
      {:else if mappings.length === 0}
        <div class="py-8 text-center">
          <Link2 class="mx-auto mb-4 h-12 w-12 text-muted-foreground" />
          <p class="mb-4 text-muted-foreground">
            No framework IDE mappings configured
          </p>
          <Button onclick={startAdding}>
            <Plus class="mr-2 h-4 w-4" />
            Create Your First Mapping
          </Button>
        </div>
      {:else}
        <div class="space-y-3">
          {#each mappings as mapping}
            <div
              class="flex items-center justify-between rounded-md border p-4 transition-colors hover:bg-accent"
            >
              <div class="flex items-center gap-4">
                <Badge variant="secondary" class="font-medium"
                  >{mapping.framework}</Badge
                >
                <ArrowRight class="h-4 w-4 text-muted-foreground" />
                <span class="text-sm font-medium"
                  >{getIdeName(mapping.ide_id)}</span
                >
              </div>

              <div class="flex items-center gap-2">
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => startEditing(mapping)}
                >
                  <Edit class="h-4 w-4" />
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onclick={() => deleteMapping(mapping)}
                  class="text-destructive hover:text-destructive"
                >
                  <Trash2 class="h-4 w-4" />
                </Button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </CardContent>
  </Card>

  <!-- Mapped Frameworks Summary -->
  {#if mappings.length > 0}
    <Card>
      <CardHeader>
        <CardTitle>Mapped Frameworks</CardTitle>
      </CardHeader>
      <CardContent>
        <div class="flex flex-wrap gap-2">
          {#each mappings as mapping}
            <Badge variant="default" class="gap-1">
              {mapping.framework}
              <ArrowRight class="h-3 w-3" />
              {getIdeName(mapping.ide_id)}
            </Badge>
          {/each}
        </div>
      </CardContent>
    </Card>
  {/if}
</div>

<!-- Add/Edit Modal -->
<Dialog.Root bind:open={showModal}>
  <Dialog.Content class="max-w-md">
    <Dialog.Header>
      <Dialog.Title>
        {editingMapping
          ? "Edit Framework IDE Mapping"
          : "Add Framework IDE Mapping"}
      </Dialog.Title>
    </Dialog.Header>

    <div class="space-y-4">
        <div class="space-y-2">
          <Label for="mapping-framework">Framework</Label>
          <Input
            id="mapping-framework"
            bind:value={mappingFramework}
            placeholder="e.g., React, Vue, Angular, Node.js"
            required
          />
          <p class="text-xs text-muted-foreground">
            The framework name that will trigger this IDE
          </p>
        </div>

        <div class="space-y-2">
          <Label for="mapping-ide">IDE</Label>
          <Select
            defaultValue={mappingIdeId || ""}
            options={ides.map((ide) => ({
              value: String(ide.id || ""),
              label: ide.name,
            }))}
            onSelect={(value) => (mappingIdeId = value || "")}
            placeholder="Select an IDE..."
          />
          <p class="text-xs text-muted-foreground">
            The IDE that will open for this framework
          </p>
        </div>

        <div class="divider-edge-t divider-edge-full flex justify-end gap-3 pt-4">
          <Button variant="outline" onclick={closeModal}>Cancel</Button>
          <Button onclick={saveMapping}
            >{editingMapping ? "Update" : "Create"} Mapping</Button
          >
        </div>
    </div>
  </Dialog.Content>
</Dialog.Root>
