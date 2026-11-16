<script lang="ts">
  import { page } from '$app/state';
  import { DEFAULT_AGENT_BACKEND } from '$lib/config/agents-tabs';
  import AgentsWorkspace from '$lib/domains/agents/components/AgentsWorkspace.svelte';
</script>

<svelte:head>
  <title>Agents - Portal Desktop</title>
</svelte:head>

<AgentsWorkspace backendId={page.params.backend ?? DEFAULT_AGENT_BACKEND} />
