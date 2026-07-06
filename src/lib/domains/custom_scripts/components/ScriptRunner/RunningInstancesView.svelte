<!--
	Running Instances View Component
	Shows tabs for all running instances of a script with their outputs and inputs
-->

<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import {
    Card,
    CardContent,
    CardDescription,
    CardHeader,
    CardTitle,
  } from "$lib/components/ui/card";
  import { Badge } from "$lib/components/ui/badge";
  import { Input } from "$lib/components/ui/input";
  import {
    Tabs,
    TabsContent,
    TabsList,
    TabsTrigger,
  } from "$lib/components/ui/tabs";
  import { Square, X } from "@lucide/svelte";
  import {
    useRunningScripts,
    type RunningScript,
  } from "../../hooks/useRunningScripts";
  import { TerminalService } from "$lib/domains/terminal/services/terminalService";
  import type { TerminalOutput } from "$lib/domains/terminal/types";

  interface Props {
    scriptId: number;
    onClose: () => void;
  }

  let { scriptId, onClose }: Props = $props();

  const runningScripts = useRunningScripts();
  let runningInstances = $state<RunningScript[]>([]);
  let activeTab = $state<string>("");

  // Subscribe to running scripts changes
  $effect(() => {
    const unsubscribe = runningScripts.subscribe((scripts) => {
      runningInstances = scripts.filter((s) => s.scriptId === scriptId);
      // Set active tab to first instance if not set
      if (runningInstances.length > 0 && !activeTab) {
        activeTab = runningInstances[0].id;
      }
      // If active tab instance no longer exists, switch to first available
      if (activeTab && !runningInstances.find((s) => s.id === activeTab)) {
        activeTab = runningInstances.length > 0 ? runningInstances[0].id : "";
      }
    });
    return unsubscribe;
  });

  // Subscribe to output for each instance (only if not already subscribed)
  // Note: We don't unsubscribe when this component unmounts because the scripts are still running
  // The subscriptions will be cleaned up when the scripts actually stop
  $effect(() => {
    for (const instance of runningInstances) {
      // Only subscribe if not already subscribed
      if (!instance.outputUnsubscribe) {
        TerminalService.subscribeToOutput(
          instance.processId,
          (output: TerminalOutput) => {
            runningScripts.appendOutput(instance.id, output.content);
          },
        )
          .then((unsub) => {
            // Update instance with unsubscribe function
            const updated = runningScripts.getById(instance.id);
            if (updated) {
              updated.outputUnsubscribe = unsub;
            }
          })
          .catch(console.error);
      }
    }
    // No cleanup here - subscriptions persist even when dialog closes
  });

  async function handleStop(instanceId: string) {
    const instance = runningScripts.getById(instanceId);
    if (instance) {
      await instance.stopCallback();
    }
  }

  function sendInput(instanceId: string, input: string) {
    const instance = runningScripts.getById(instanceId);
    if (instance) {
      TerminalService.sendInput(instance.processId, input).catch(console.error);
    }
  }

  const activeInstance = $derived(
    runningInstances.find((i) => i.id === activeTab),
  );
  let activeOutputContainer = $state<HTMLDivElement | null>(null);

  // Auto-scroll output when it updates
  $effect(() => {
    if (activeInstance && activeInstance.output && activeOutputContainer) {
      requestAnimationFrame(() => {
        if (activeOutputContainer) {
          activeOutputContainer.scrollTop = activeOutputContainer.scrollHeight;
        }
      });
    }
  });
</script>

<Dialog.Root
  open={true}
  onOpenChange={(isOpen) => {
    if (!isOpen) onClose();
  }}
>
  <Dialog.Content
    class="flex max-h-[90vh] max-w-6xl flex-col overflow-hidden p-0"
  >
    <div class="flex items-center justify-between border-b p-6">
      <div>
        <Dialog.Title class="flex items-center gap-2 text-2xl">
          {#if activeInstance?.script.icon}
            <span>{activeInstance.script.icon}</span>
          {/if}
          {activeInstance?.script.name || "Running Scripts"}
        </Dialog.Title>
        {#if activeInstance?.script.description}
          <p class="mt-1 text-sm text-muted-foreground">
            {activeInstance.script.description}
          </p>
        {/if}
      </div>
      <Button variant="ghost" size="sm" onclick={onClose}>
        <X class="h-4 w-4" />
      </Button>
    </div>

    <div class="flex-1 overflow-hidden p-6">
      {#if runningInstances.length === 0}
        <div class="py-12 text-center">
          <p class="text-muted-foreground">No running instances</p>
        </div>
      {:else}
        <Tabs
          value={activeTab}
          onValueChange={(value) => (activeTab = value)}
          class="flex h-full flex-col"
        >
          <TabsList class="mb-4">
            {#each runningInstances as instance (instance.id)}
              <TabsTrigger value={instance.id}>
                Instance {runningInstances.indexOf(instance) + 1}
                <Badge variant="outline" class="ml-2">
                  {new Date(instance.startTime).toLocaleTimeString()}
                </Badge>
              </TabsTrigger>
            {/each}
          </TabsList>

          {#each runningInstances as instance (instance.id)}
            <TabsContent
              value={instance.id}
              class="mt-0 flex flex-1 flex-col overflow-hidden"
            >
              {@const isActive = activeTab === instance.id}
              <Card class="flex flex-1 flex-col overflow-hidden">
                <CardHeader>
                  <div class="flex items-center justify-between">
                    <CardTitle>Output</CardTitle>
                    <Button
                      variant="destructive"
                      size="sm"
                      onclick={() => handleStop(instance.id)}
                    >
                      <Square class="mr-2 h-4 w-4" />
                      Stop
                    </Button>
                  </div>
                </CardHeader>
                <CardContent
                  class="flex flex-1 flex-col space-y-4 overflow-hidden"
                >
                  {#if isActive}
                    <div
                      bind:this={activeOutputContainer}
                      class="flex-1 overflow-y-auto rounded-md bg-black p-4 font-mono text-sm text-green-400"
                    >
                      {instance.output || "Waiting for output..."}
                    </div>
                  {:else}
                    <div
                      class="flex-1 overflow-y-auto rounded-md bg-black p-4 font-mono text-sm text-green-400"
                    >
                      {instance.output || "Waiting for output..."}
                    </div>
                  {/if}

                  <div class="flex gap-2">
                    <Input
                      type="text"
                      placeholder="Type input and press Enter (e.g., password for sudo)..."
                      class="flex-1"
                      autofocus
                      onkeydown={(e) => {
                        if (e.key === "Enter") {
                          const input =
                            (e.target as HTMLInputElement).value + "\n";
                          sendInput(instance.id, input);
                          (e.target as HTMLInputElement).value = "";
                        }
                      }}
                    />
                  </div>
                </CardContent>
              </Card>
            </TabsContent>
          {/each}
        </Tabs>
      {/if}
    </div>
  </Dialog.Content>
</Dialog.Root>
