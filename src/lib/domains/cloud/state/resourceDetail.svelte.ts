// Load-by-name + loading/error + tab-sync-from-URL shared by the cloud detail pages.
// Build once in component init: it registers an $effect.
import { get } from 'svelte/store';
import { page } from '$app/state';
import { replaceState } from '$app/navigation';
import { cloudStore, loadResources } from '$lib/domains/cloud/stores';
import type { ICloudResource, ResourceType } from '$lib/domains/cloud/core/types';

export class ResourceDetail {
  resource = $state<ICloudResource | null>(null);
  isLoading = $state(true);
  error = $state<string | null>(null);
  activeTab = $state('overview');

  // getter, not $derived: get(cloudStore) is read fresh on every call
  get namespace() {
    return page.url.searchParams.get('namespace') || get(cloudStore).selectedNamespace || 'default';
  }
  #tabParam = $derived(page.url.searchParams.get('tab') || 'overview');

  /** `kind` is the display name used in messages, e.g. "ConfigMap". */
  constructor(
    private type: ResourceType,
    private kind: string,
    private name: () => string | undefined
  ) {
    $effect(() => {
      this.activeTab = this.#tabParam;
    });
  }

  load = async () => {
    const name = this.name();
    if (!name || !get(cloudStore).connection.isConnected) {
      this.error = `${this.kind} name or connection required`;
      this.isLoading = false;
      return;
    }

    try {
      this.isLoading = true;
      this.error = null;

      await loadResources(this.type, this.namespace);
      const resources = get(cloudStore).resources[this.type] || [];
      this.resource = resources.find((r) => r.name === name) || null;

      if (!this.resource) {
        this.error = `${this.kind} "${name}" not found in namespace "${this.namespace}".`;
      }
    } catch (err) {
      this.error = err instanceof Error ? err.message : `Failed to load ${this.kind}`;
      console.error(`Failed to load ${this.kind}:`, err);
    } finally {
      this.isLoading = false;
    }
  };

  setTab = (tab: string) => {
    this.activeTab = tab;
    const url = new URL(page.url.href);
    url.searchParams.set('tab', tab);
    replaceState(url, {});
  };
}
