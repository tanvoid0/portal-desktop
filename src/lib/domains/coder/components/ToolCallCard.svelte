<script lang="ts">
  import { Badge } from "$lib/components/ui/badge";
  import { ChevronRight, Wrench } from "@lucide/svelte";
  import type { ToolCall } from "../types.js";

  interface Props {
    call: ToolCall;
    /** The matching `tool` message content, if the call already ran. */
    result?: string | null;
  }

  let { call, result = null }: Props = $props();

  let open = $state(false);

  const args = $derived.by(() => {
    try {
      return JSON.parse(call.function.arguments || "{}") as Record<string, unknown>;
    } catch {
      return {} as Record<string, unknown>;
    }
  });

  const isWrite = $derived(call.function.name === "write_file");
  const writeContent = $derived(isWrite ? String(args.content ?? "") : "");
  const isEdit = $derived(call.function.name === "edit_file");
</script>

<div class="rounded-md border border-border bg-muted/40 text-sm">
  <button
    type="button"
    class="flex w-full items-center gap-2 px-3 py-2 text-left"
    onclick={() => (open = !open)}
  >
    <ChevronRight
      class="h-3.5 w-3.5 shrink-0 transition-transform {open ? 'rotate-90' : ''}"
    />
    <Wrench class="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
    <span class="font-mono text-xs font-medium">{call.function.name}</span>
    {#if result !== null}
      <Badge variant="secondary" class="ml-auto text-[10px]">done</Badge>
    {:else}
      <Badge variant="outline" class="ml-auto text-[10px]">pending</Badge>
    {/if}
  </button>

  {#if open}
    <div class="space-y-2 border-t border-border px-3 py-2">
      {#if isWrite}
        <div>
          <div class="mb-1 text-xs text-muted-foreground">
            {String(args.path ?? "")}
          </div>
          <pre class="max-h-64 overflow-auto rounded bg-background p-2 text-xs"><code
              >{writeContent}</code
            ></pre>
        </div>
      {:else if isEdit}
        <div class="space-y-1">
          <div class="text-xs text-muted-foreground">{String(args.path ?? "")}</div>
          <pre class="max-h-40 overflow-auto rounded bg-red-500/10 p-2 text-xs"><code
              >- {String(args.old_string ?? "")}</code
            ></pre>
          <pre class="max-h-40 overflow-auto rounded bg-green-500/10 p-2 text-xs"><code
              >+ {String(args.new_string ?? "")}</code
            ></pre>
        </div>
      {:else}
        <pre class="overflow-auto rounded bg-background p-2 text-xs"><code
            >{JSON.stringify(args, null, 2)}</code
          ></pre>
      {/if}

      {#if result !== null}
        <div>
          <div class="mb-1 text-xs text-muted-foreground">result</div>
          <pre
            class="max-h-64 overflow-auto rounded bg-background p-2 text-xs"><code
              >{result}</code
            ></pre>
        </div>
      {/if}
    </div>
  {/if}
</div>
