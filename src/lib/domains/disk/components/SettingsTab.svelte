<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import type { AiConfig, TeamOption } from "../types";
  import { loadAiConfig, saveAiConfig } from "../utils";

  let cfg = $state<AiConfig>(loadAiConfig());
  let status = $state("");
  let teams = $state<TeamOption[] | null>(null);
  let teamsError = $state("");
  let teamsBusy = $state(false);

  const appTeam = $derived(teams?.find((t) => t.isAppTeam));

  const btnPrimary =
    "inline-flex items-center justify-center gap-1.5 h-9 px-4 rounded-md bg-white text-black text-sm font-medium transition-colors hover:bg-neutral-200 disabled:opacity-40 disabled:pointer-events-none";
  const inputCls =
    "h-9 px-3 rounded-md bg-neutral-950 border border-neutral-800 text-sm text-neutral-100 placeholder:text-neutral-600 transition-colors focus:outline-none focus:border-neutral-600";

  function normalize(c: AiConfig): AiConfig {
    return {
      baseUrl: c.baseUrl?.trim() || undefined,
      apiToken: c.apiToken?.trim() || undefined,
      teamTemplateId: c.teamTemplateId === undefined || Number.isNaN(c.teamTemplateId) ? undefined : c.teamTemplateId,
    };
  }

  async function loadTeams(current: AiConfig = cfg) {
    teamsBusy = true;
    teamsError = "";
    try {
      teams = await invoke<TeamOption[]>("list_ai_teams", { config: current });
    } catch (e) {
      teams = null;
      teamsError = String(e);
    } finally {
      teamsBusy = false;
    }
  }

  async function createTeam() {
    teamsBusy = true;
    teamsError = "";
    try {
      const team = await invoke<TeamOption>("provision_ai_team", { config: cfg });
      const cleaned = { ...normalize(cfg), teamTemplateId: team.id };
      saveAiConfig(cleaned);
      cfg = cleaned;
      await loadTeams(cleaned);
      status = `Created "${team.name}" (id ${team.id}) and selected it.`;
    } catch (e) {
      teamsError = String(e);
    } finally {
      teamsBusy = false;
    }
  }

  onMount(() => {
    if (cfg.baseUrl || cfg.apiToken) void loadTeams();
  });

  function save() {
    const cleaned = normalize(cfg);
    saveAiConfig(cleaned);
    cfg = cleaned;
    status = "Saved.";
  }
</script>

<div class="mb-4 rounded-md border border-blue-500/30 bg-blue-500/10 p-3 text-xs text-blue-300/90">
  AI verification now runs through Portal's own <a href="/ai/providers" class="underline">AI providers</a>
  (Ollama, Gemini, …). Configure and enable a provider there — no separate token needed. The Agent
  Platform settings below are legacy and only affect the (currently unused) team-based path.
</div>

<p class="mb-4 max-w-2xl text-sm text-neutral-500">
  <span class="text-neutral-400">Legacy — Agent Platform.</span> Point it at your instance and
  authenticate with a <span class="text-neutral-300">project-scoped token</span> (starts with
  <code class="text-neutral-300">agp_</code>) minted from the platform dashboard — not the master
  key. The token needs scopes <code class="text-neutral-300">process:read</code> and
  <code class="text-neutral-300">process:write</code>.
</p>

<div class="overflow-hidden rounded-xl border border-neutral-800 bg-neutral-950/60">
  <div class="max-w-xl space-y-4 p-4">
    <label class="block">
      <div class="mb-1.5 text-xs uppercase tracking-wide text-neutral-500">Base URL</div>
      <input bind:value={cfg.baseUrl} placeholder="http://127.0.0.1:18410" class="w-full {inputCls}" />
    </label>

    <label class="block">
      <div class="mb-1.5 text-xs uppercase tracking-wide text-neutral-500">API token (agp_…)</div>
      <input type="password" bind:value={cfg.apiToken} placeholder="agp_…" class="w-full {inputCls} font-mono" />
    </label>

    <div class="block">
      <div class="mb-1.5 flex items-center justify-between">
        <div class="text-xs uppercase tracking-wide text-neutral-500">Verification team</div>
        <button type="button" onclick={() => void loadTeams()} disabled={teamsBusy} class="text-xs text-neutral-400 hover:text-neutral-200 disabled:opacity-50">{teamsBusy ? "Loading…" : "Refresh"}</button>
      </div>
      <select
        value={cfg.teamTemplateId ?? (appTeam ? appTeam.id : "")}
        onchange={(e) => (cfg.teamTemplateId = e.currentTarget.value === "" ? undefined : Number(e.currentTarget.value))}
        disabled={!teams || teams.length === 0}
        class="w-full {inputCls}"
      >
        <option value="">{appTeam ? `Auto — ${appTeam.name} (id ${appTeam.id})` : "Auto — app team"}</option>
        {#each teams ?? [] as t (t.id)}
          <option value={t.id}>{t.name} (id {t.id}){t.isAppTeam ? " ✓ app team" : ""}</option>
        {/each}
      </select>

      {#if teamsError}<p class="mt-1.5 text-xs text-red-400">Couldn't load teams: {teamsError}</p>{/if}

      {#if teams && !appTeam}
        <div class="mt-2 rounded-md border border-amber-500/30 bg-amber-500/10 p-3">
          <p class="mb-2 text-xs text-amber-300/90">No matching <span class="font-medium">Deletion Verifier</span> team found on this platform. Verification needs the app's roster (Risk Assessor, Data Preservation Checker, Lead Synthesizer).</p>
          <button type="button" onclick={() => void createTeam()} disabled={teamsBusy} class="{btnPrimary} text-xs disabled:opacity-50">{teamsBusy ? "Creating…" : "Create team for me"}</button>
        </div>
      {/if}
      {#if teams && appTeam}
        <p class="mt-1.5 text-xs text-neutral-500">App team present. Leave on "Auto" to always use it.</p>
      {/if}
    </div>

    <div class="flex items-center gap-3 pt-1">
      <button onclick={save} class={btnPrimary}>Save</button>
      {#if status}<span class="text-sm text-neutral-500">{status}</span>{/if}
    </div>
  </div>
</div>
