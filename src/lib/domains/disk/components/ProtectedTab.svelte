<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { Card } from "$lib/components/ui/card";
  import {
    Table,
    TableBody,
    TableCell,
    TableHead,
    TableHeader,
    TableRow,
  } from "$lib/components/ui/table";

  let { active }: { active: boolean } = $props();

  let paths = $state<string[]>([]);
  let input = $state("");
  let status = $state("");

  async function load() {
    try {
      paths = await invoke<string[]>("list_protected");
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
      await invoke("add_protected", { path: p });
      input = "";
      status = `Protected: ${p}`;
      load();
    } catch (e) {
      status = `Add failed: ${String(e)}`;
    }
  }

  async function pickFolder() {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === "string") add(dir);
  }

  async function remove(path: string) {
    try {
      await invoke("remove_protected", { path });
      status = `Removed: ${path}`;
      load();
    } catch (e) {
      status = `Remove failed: ${String(e)}`;
    }
  }
</script>

<p class="mb-4 max-w-2xl text-sm text-muted-foreground">
  Paths listed here are never proposed for cleanup — anything under them is skipped during scans. OS
  and program folders are already protected by default.
</p>

<div class="mb-4 flex items-center gap-2">
  <Button variant="outline" onclick={pickFolder}>Add folder…</Button>
  <Input
    bind:value={input}
    onkeydown={(e) => e.key === "Enter" && add(input)}
    placeholder="C:\Users\you\Projects"
    class="flex-1"
  />
  <Button onclick={() => add(input)} disabled={!input.trim()}>Protect</Button>
</div>

{#if status}<div class="mb-4 text-sm text-muted-foreground">{status}</div>{/if}

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
            <Button variant="outline" size="sm" onclick={() => remove(p)} class="text-destructive hover:text-destructive">
              Remove
            </Button>
          </TableCell>
        </TableRow>
      {/each}
      {#if paths.length === 0}
        <TableRow>
          <TableCell colspan={2} class="py-10 text-center text-muted-foreground">
            No custom protected paths. Add folders you never want touched.
          </TableCell>
        </TableRow>
      {/if}
    </TableBody>
  </Table>
</Card>
