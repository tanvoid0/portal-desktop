<!--
	Rules that hold across every scan: what is never touched, and where the
	optional AI review gets its credentials. Both were their own tab and neither
	filled one — they are settings, and settings belong together.
-->

<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { AI_PROVIDER_SETTINGS_PATH } from '$lib/config/ai-nav';
  import { Button } from '$lib/components/ui/button';
  import { Input } from '$lib/components/ui/input';
  import { Card, CardContent, CardHeader, CardTitle } from '$lib/components/ui/card';
  import { FolderOpen, ShieldCheck, Sparkles } from '@lucide/svelte';
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from '$lib/components/ui/table';

  let { active }: { active: boolean } = $props();

  let paths = $state<string[]>([]);
  let input = $state('');
  let status = $state('');

  async function load() {
    try {
      paths = await invoke<string[]>('list_protected');
    } catch (e) {
      status = `Failed to load: ${String(e)}`;
    }
  }

  let lastActive = false;
  $effect(() => {
    if (active && !lastActive) void load();
    lastActive = active;
  });

  async function add(path: string) {
    const p = path.trim();
    if (!p) return;
    try {
      await invoke('add_protected', { path: p });
      input = '';
      status = `Protected: ${p}`;
      load();
    } catch (e) {
      status = `Add failed: ${String(e)}`;
    }
  }

  async function pickFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === 'string') add(dir);
  }

  async function remove(path: string) {
    try {
      await invoke('remove_protected', { path });
      status = `Removed: ${path}`;
      load();
    } catch (e) {
      status = `Remove failed: ${String(e)}`;
    }
  }
</script>

<div class="space-y-4">
  <Card class="gap-0 overflow-hidden py-0">
    <CardHeader class="divider-edge-b divider-edge-full bg-muted/40 px-4 py-3 [.border-b]:pb-3">
      <CardTitle class="flex items-center gap-2 text-sm font-medium">
        <ShieldCheck class="size-4" /> Never touch these folders
      </CardTitle>
    </CardHeader>
    <CardContent class="space-y-3 p-4">
      <p class="max-w-2xl text-sm text-muted-foreground">
        Anything under a protected path is skipped by every scan, so it can never be proposed or
        removed. Windows, Program Files and the other system folders are protected already — this
        list is for your own work.
      </p>

      <div class="flex items-center gap-2">
        <Button variant="outline" onclick={pickFolder}>
          <FolderOpen class="size-4" /> Add folder…
        </Button>
        <Input
          bind:value={input}
          onkeydown={(e) => e.key === 'Enter' && add(input)}
          placeholder="C:\Users\you\Projects"
          class="flex-1"
        />
        <Button onclick={() => add(input)} disabled={!input.trim()}>Protect</Button>
      </div>

      {#if status}<div class="text-sm text-muted-foreground">{status}</div>{/if}

      <Card class="gap-0 overflow-hidden py-0">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Protected path</TableHead>
              <TableHead class="w-24"></TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {#each paths as p (p)}
              <TableRow>
                <TableCell class="break-all font-mono text-xs">{p}</TableCell>
                <TableCell class="text-right">
                  <Button
                    variant="outline"
                    size="sm"
                    onclick={() => remove(p)}
                    class="text-destructive hover:text-destructive"
                  >
                    Remove
                  </Button>
                </TableCell>
              </TableRow>
            {/each}
            {#if paths.length === 0}
              <TableRow>
                <TableCell colspan={2} class="py-10 text-center text-muted-foreground">
                  No custom protected paths yet. Add folders you never want touched.
                </TableCell>
              </TableRow>
            {/if}
          </TableBody>
        </Table>
      </Card>
    </CardContent>
  </Card>

  <Card class="gap-0 overflow-hidden py-0">
    <CardHeader class="divider-edge-b divider-edge-full bg-muted/40 px-4 py-3 [.border-b]:pb-3">
      <CardTitle class="flex items-center gap-2 text-sm font-medium">
        <Sparkles class="size-4" /> AI second opinion
      </CardTitle>
    </CardHeader>
    <CardContent class="max-w-2xl space-y-3 p-4">
      <p class="text-sm text-muted-foreground">
        On the Files tab, <span class="text-foreground">Ask AI to double-check</span> sends the proposal
        list for review. It is advisory only — it can flag an item, never tick or remove one.
      </p>
      <p class="text-sm text-muted-foreground">
        It uses the desktop app's configured provider — no separate disk-utility credentials. Set
        base URL, API token and model under
        <Button variant="link" href={AI_PROVIDER_SETTINGS_PATH} class="h-auto p-0">
          Settings → AI Providers
        </Button>. Backend providers and models are configured on agent-platform (<code
          class="text-muted-foreground">/config</code
        >), not here.
      </p>
    </CardContent>
  </Card>
</div>
