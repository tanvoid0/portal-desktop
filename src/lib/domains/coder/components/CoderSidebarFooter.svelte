<script lang="ts">
  import { Button } from "$lib/components/ui/button";
  import { Badge } from "$lib/components/ui/badge";
  import { Settings2, Sparkles, Trash2 } from "@lucide/svelte";
  import { goto } from "$app/navigation";
  import { AI_PROVIDER_SETTINGS_PATH } from "$lib/config/ai-nav";
  import { coderSession } from "../state/coderSession.svelte.js";
  import type { PermissionMode, PermissionRule } from "../types.js";

  interface Props {
    showRules?: boolean;
    onToggleRules?: () => void;
  }

  let { showRules = false, onToggleRules }: Props = $props();

  const MODES: { value: PermissionMode; label: string }[] = [
    { value: "review", label: "Review" },
    { value: "auto-accept-all", label: "Auto" },
    { value: "plan", label: "Plan" },
  ];

  function removeRule(r: PermissionRule) {
    coderSession.removeRule(r);
  }
</script>

<div class="divider-edge-t divider-edge-full shrink-0 space-y-2 bg-muted/20 px-3 py-2.5">
  <div class="flex items-center justify-between gap-2">
    <span class="text-[11px] font-semibold uppercase tracking-wide text-muted-foreground">
      Settings
    </span>
    <div class="flex items-center gap-0.5">
      <Button
        size="icon"
        variant="ghost"
        class="h-6 w-6"
        title="AI provider settings"
        onclick={() => goto(AI_PROVIDER_SETTINGS_PATH)}
      >
        <Sparkles class="h-3.5 w-3.5" />
      </Button>
      <Button
        size="icon"
        variant="ghost"
        class="h-6 w-6"
        title="Permission rules"
        onclick={() => onToggleRules?.()}
      >
        <Settings2 class="h-3.5 w-3.5" />
      </Button>
    </div>
  </div>

  <div class="flex flex-wrap gap-1">
    {#each MODES as m}
      <Button
        size="sm"
        variant={coderSession.mode === m.value ? "secondary" : "ghost"}
        class="h-6 px-2 text-[11px]"
        onclick={() => void coderSession.changeMode(m.value)}
      >
        {m.label}
      </Button>
    {/each}
  </div>

  {#if showRules}
    <div class="rounded border border-border/60 bg-background p-2 text-xs">
      <div class="mb-1.5 font-medium">Allow / deny rules</div>
      {#if coderSession.rules.length === 0}
        <p class="text-muted-foreground">
          No saved rules. "Accept &amp; remember" on an approval adds one.
        </p>
      {:else}
        <ul class="max-h-32 space-y-1 overflow-y-auto">
          {#each coderSession.rules as r}
            <li class="flex items-center gap-1.5">
              <Badge variant={r.allow ? "secondary" : "destructive"} class="text-[10px]">
                {r.allow ? "allow" : "deny"}
              </Badge>
              <span class="truncate font-mono">{r.tool}</span>
              <span class="truncate font-mono text-muted-foreground">{r.pattern || "*"}</span>
              <Button
                type="button"
                variant="ghost"
                size="icon-sm"
                class="ml-auto h-6 w-6 shrink-0 text-muted-foreground hover:text-destructive"
                onclick={() => removeRule(r)}
                title="Remove rule"
              >
                <Trash2 class="h-3 w-3" />
              </Button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>
