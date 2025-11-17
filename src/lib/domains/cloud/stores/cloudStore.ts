// Cloud store with provider-agnostic state management
// Uses Svelte stores for reactive state

import { writable, derived, get } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type {
  ICluster,
  ICloudResource
} from '../core/types';
import {
  CloudProviderType,
  ResourceType,
  ClusterStatus
} from '../core/types';
import { CloudServiceFactory } from '../services/cloudServiceFactory';
import { toastActions } from '$lib/domains/shared/stores/toastStore';
import type { PodInfo } from '../providers/gcp/GCPTypes';
import { GCPProvider } from '../providers/gcp/GCPProvider';

// Storage keys
const STORAGE_KEY_PREFIX = 'portal-cloud-';
const STORAGE_KEY_NAMESPACE = `${STORAGE_KEY_PREFIX}selected-namespace`;
const STORAGE_KEY_CLUSTER_ID = `${STORAGE_KEY_PREFIX}last-cluster-id`;
const STORAGE_KEY_PROVIDER = `${STORAGE_KEY_PREFIX}last-provider`;

// Persistence helpers
function saveToStorage(key: string, value: string | null): void {
  if (typeof window === 'undefined') return;
  
  try {
    if (value === null || value === '') {
      localStorage.removeItem(key);
    } else {
      localStorage.setItem(key, value);
    }
  } catch (error) {
    console.warn(`Failed to save ${key} to localStorage:`, error);
  }
}

function loadFromStorage(key: string, defaultValue: string = ''): string {
  if (typeof window === 'undefined') return defaultValue;
  
  try {
    return localStorage.getItem(key) || defaultValue;
  } catch (error) {
    console.warn(`Failed to load ${key} from localStorage:`, error);
    return defaultValue;
  }
}

export interface CloudState {
  // Provider-agnostic state
  currentProvider: CloudProviderType | null;
  currentCluster: ICluster | null;
  selectedNamespace: string;
  
  // Connection state
  connection: {
    isConnected: boolean;
    isConnecting: boolean;
    error: string | null;
  };
  
  // Resource state
  resources: {
    [ResourceType.POD]: ICloudResource[];
    [ResourceType.SERVICE]: ICloudResource[];
    [ResourceType.DEPLOYMENT]: ICloudResource[];
    [ResourceType.STATEFULSET]: ICloudResource[];
    [ResourceType.DAEMONSET]: ICloudResource[];
    [ResourceType.JOB]: ICloudResource[];
    [ResourceType.CRONJOB]: ICloudResource[];
    [ResourceType.CONFIGMAP]: ICloudResource[];
    [ResourceType.SECRET]: ICloudResource[];
    [ResourceType.INGRESS]: ICloudResource[];
    [ResourceType.NAMESPACE]: ICloudResource[];
  };
  
  // Loading states
  loading: {
    clusters: boolean;
    resources: Record<ResourceType, boolean>;
  };
}

// Load persisted state from localStorage
const persistedNamespace = loadFromStorage(STORAGE_KEY_NAMESPACE, '');
const persistedClusterId = loadFromStorage(STORAGE_KEY_CLUSTER_ID, '');
const persistedProvider = loadFromStorage(STORAGE_KEY_PROVIDER, '') as CloudProviderType | '';

const initialState: CloudState = {
  currentProvider: persistedProvider && Object.values(CloudProviderType).includes(persistedProvider as CloudProviderType)
    ? (persistedProvider as CloudProviderType)
    : null,
  currentCluster: persistedClusterId ? { id: persistedClusterId } as ICluster : null,
  selectedNamespace: persistedNamespace,
  connection: {
    isConnected: false,
    isConnecting: false,
    error: null
  },
  resources: {
    [ResourceType.POD]: [],
    [ResourceType.SERVICE]: [],
    [ResourceType.DEPLOYMENT]: [],
    [ResourceType.STATEFULSET]: [],
    [ResourceType.DAEMONSET]: [],
    [ResourceType.JOB]: [],
    [ResourceType.CRONJOB]: [],
    [ResourceType.CONFIGMAP]: [],
    [ResourceType.SECRET]: [],
    [ResourceType.INGRESS]: [],
    [ResourceType.NAMESPACE]: []
  },
  loading: {
    clusters: false,
    resources: {
      [ResourceType.POD]: false,
      [ResourceType.SERVICE]: false,
      [ResourceType.DEPLOYMENT]: false,
      [ResourceType.STATEFULSET]: false,
      [ResourceType.DAEMONSET]: false,
      [ResourceType.JOB]: false,
      [ResourceType.CRONJOB]: false,
      [ResourceType.CONFIGMAP]: false,
      [ResourceType.SECRET]: false,
      [ResourceType.INGRESS]: false,
      [ResourceType.NAMESPACE]: false
    }
  }
};

export const cloudStore = writable<CloudState>(initialState);

// Subscribe to store changes to persist namespace and cluster
cloudStore.subscribe((state) => {
  // Persist selected namespace
  saveToStorage(STORAGE_KEY_NAMESPACE, state.selectedNamespace);
  
  // Persist cluster ID and provider when connected
  if (state.currentCluster && state.currentProvider) {
    saveToStorage(STORAGE_KEY_CLUSTER_ID, state.currentCluster.id);
    saveToStorage(STORAGE_KEY_PROVIDER, state.currentProvider);
  } else {
    // Clear persisted cluster when disconnected
    saveToStorage(STORAGE_KEY_CLUSTER_ID, null);
    saveToStorage(STORAGE_KEY_PROVIDER, null);
  }
});

// Derived stores for convenience
export const isConnected = derived(cloudStore, $store => $store.connection.isConnected);
export const currentCluster = derived(cloudStore, $store => $store.currentCluster);
export const currentProvider = derived(cloudStore, $store => $store.currentProvider);
export const selectedNamespace = derived(cloudStore, $store => $store.selectedNamespace);

// Store actions

/**
 * Connect to a cluster using the specified provider
 */
export async function connectToCluster(
  providerType: CloudProviderType,
  clusterId: string
): Promise<void> {
  const state = get(cloudStore);
  
  // Update connecting state
  cloudStore.update(s => ({
    ...s,
    connection: { ...s.connection, isConnecting: true, error: null }
  }));
  
  try {
    const provider = await CloudServiceFactory.getProvider(providerType);
    await provider.connect(clusterId);
    const cluster = await provider.getCluster(clusterId);
    
    if (!cluster) {
      throw new Error(`Cluster ${clusterId} not found`);
    }
    
    // Restore persisted namespace if available, otherwise use cluster default
    const persistedNamespace = loadFromStorage(STORAGE_KEY_NAMESPACE, '');
    const namespaceToUse = persistedNamespace || cluster.namespace || '';
    
    cloudStore.update(s => ({
      ...s,
      currentProvider: providerType,
      currentCluster: cluster,
      connection: {
        isConnected: true,
        isConnecting: false,
        error: null
      },
      selectedNamespace: namespaceToUse
    }));
    
    // Start watching for real-time updates
    await startWatchingResources(namespaceToUse || undefined);
    
    toastActions.success(`Connected to cluster: ${cluster.name}`);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Failed to connect to cluster';
    
    cloudStore.update(s => ({
      ...s,
      connection: {
        isConnected: false,
        isConnecting: false,
        error: errorMessage
      }
    }));
    
    toastActions.error(errorMessage);
    throw error;
  }
}

/**
 * Disconnect from current cluster
 */
export async function disconnectFromCluster(): Promise<void> {
  const state = get(cloudStore);
  
  if (!state.currentProvider || !state.connection.isConnected) {
    return;
  }
  
  try {
    // Stop watching
    await stopWatchingResources();
    
    const provider = await CloudServiceFactory.getProvider(state.currentProvider);
    await provider.disconnect();
    
    cloudStore.update(s => ({
      ...s,
      currentProvider: null,
      currentCluster: null,
      connection: {
        isConnected: false,
        isConnecting: false,
        error: null
      },
      resources: {
        [ResourceType.POD]: [],
        [ResourceType.SERVICE]: [],
        [ResourceType.DEPLOYMENT]: [],
        [ResourceType.STATEFULSET]: [],
        [ResourceType.DAEMONSET]: [],
        [ResourceType.JOB]: [],
        [ResourceType.CRONJOB]: [],
        [ResourceType.CONFIGMAP]: [],
        [ResourceType.SECRET]: [],
        [ResourceType.INGRESS]: [],
        [ResourceType.NAMESPACE]: []
      }
    }));
    
    // Clear persisted cluster info
    saveToStorage(STORAGE_KEY_CLUSTER_ID, null);
    saveToStorage(STORAGE_KEY_PROVIDER, null);
    
    toastActions.success('Disconnected from cluster');
  } catch (error) {
    console.error('Failed to disconnect:', error);
    toastActions.error('Failed to disconnect from cluster');
  }
}

/**
 * Load clusters for a provider
 */
export async function loadClusters(providerType: CloudProviderType): Promise<ICluster[]> {
  cloudStore.update(s => ({
    ...s,
    loading: { ...s.loading, clusters: true }
  }));
  
  try {
    const provider = await CloudServiceFactory.getProvider(providerType);
    const clusters = await provider.listClusters();
    
    cloudStore.update(s => ({
      ...s,
      loading: { ...s.loading, clusters: false }
    }));
    
    return clusters;
  } catch (error) {
    cloudStore.update(s => ({
      ...s,
      loading: { ...s.loading, clusters: false }
    }));
    
    console.error('Failed to load clusters:', error);
    toastActions.error('Failed to load clusters');
    return [];
  }
}

/**
 * Load resources of a specific type
 */
export async function loadResources(
  type: ResourceType,
  namespace?: string
): Promise<void> {
  const state = get(cloudStore);
  
  if (!state.currentProvider || !state.connection.isConnected) {
    return;
  }
  
  cloudStore.update(s => ({
    ...s,
    loading: {
      ...s.loading,
      resources: { ...s.loading.resources, [type]: true }
    }
  }));
  
  try {
    const provider = await CloudServiceFactory.getProvider(state.currentProvider);
    const resources = await provider.listResources(
      type,
      namespace || state.selectedNamespace || undefined
    );
    
    cloudStore.update(s => ({
      ...s,
      resources: {
        ...s.resources,
        [type]: resources
      },
      loading: {
        ...s.loading,
        resources: { ...s.loading.resources, [type]: false }
      }
    }));
  } catch (error) {
    cloudStore.update(s => ({
      ...s,
      loading: {
        ...s.loading,
        resources: { ...s.loading.resources, [type]: false }
      }
    }));
    
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    console.error(`Failed to load ${type} resources:`, error);
    toastActions.error(`Failed to load ${type} resources: ${errorMessage}`);
  }
}

/**
 * Set selected namespace
 */
export async function setSelectedNamespace(namespace: string): Promise<void> {
  const state = get(cloudStore);
  const previousNamespace = state.selectedNamespace;
  
  cloudStore.update(s => ({
    ...s,
    selectedNamespace: namespace
  }));
  
  // Save to storage
  saveToStorage(STORAGE_KEY_NAMESPACE, namespace);
  
  // If watching and namespace changed, restart watch
  if (isWatching && namespace !== previousNamespace && state.connection.isConnected) {
    await stopWatchingResources();
    await startWatchingResources(namespace || undefined);
  }
}

/**
 * Refresh all resources
 */
export async function refreshResources(): Promise<void> {
  const state = get(cloudStore);
  
  if (!state.currentProvider || !state.connection.isConnected) {
    return;
  }
  
  await Promise.all([
    loadResources(ResourceType.POD),
    loadResources(ResourceType.SERVICE),
    loadResources(ResourceType.DEPLOYMENT),
    loadResources(ResourceType.STATEFULSET),
    loadResources(ResourceType.DAEMONSET),
    loadResources(ResourceType.JOB),
    loadResources(ResourceType.CRONJOB),
    loadResources(ResourceType.CONFIGMAP),
    loadResources(ResourceType.SECRET),
    loadResources(ResourceType.INGRESS),
    loadResources(ResourceType.NAMESPACE)
  ]);
}

/**
 * Initialize provider (call on app startup)
 */
export async function initializeProvider(providerType: CloudProviderType): Promise<void> {
  try {
    const provider = await CloudServiceFactory.getProvider(providerType);
    await provider.initialize();
  } catch (error) {
    console.error('Failed to initialize provider:', error);
    toastActions.error('Failed to initialize cloud provider');
  }
}

// Real-time update listeners
let watchListeners: (() => void)[] = [];
let isWatching = false;
let isStoppingWatches = false; // Guard to prevent multiple simultaneous stop calls

/**
 * Start watching resources for real-time updates
 */
export async function startWatchingResources(namespace?: string): Promise<void> {
  const state = get(cloudStore);
  
  // If already watching, stop first to restart with new namespace
  if (isWatching) {
    await stopWatchingResources();
  }
  
  if (!state.connection.isConnected || state.currentProvider !== CloudProviderType.GCP) {
    return;
  }
  
  try {
    // Use selected namespace if not provided
    const watchNamespace = namespace || state.selectedNamespace || undefined;
    
    // Start backend watches for all resource types
    await Promise.all([
      invoke('k8s_start_watching_pods', { namespace: watchNamespace }),
      invoke('k8s_start_watching_services', { namespace: watchNamespace }),
      invoke('k8s_start_watching_deployments', { namespace: watchNamespace })
    ]);
    
    // Set up event listeners for pods
    const podUpdatedUnlisten = await listen<PodInfo>('k8s:pod-updated', (event) => {
      const podInfo = event.payload;
      const provider = new GCPProvider();
      const podResource = provider.mapToPod(podInfo);
      
      cloudStore.update(s => {
        const pods = s.resources[ResourceType.POD];
        const index = pods.findIndex(p => p.id === podResource.id || p.name === podResource.name);
        
        if (index >= 0) {
          // Update existing pod
          pods[index] = podResource;
        } else {
          // Add new pod
          pods.push(podResource);
        }
        
        return {
          ...s,
          resources: {
            ...s.resources,
            [ResourceType.POD]: [...pods]
          }
        };
      });
    });
    
    const podDeletedUnlisten = await listen<PodInfo>('k8s:pod-deleted', (event) => {
      const podInfo = event.payload;
      
      cloudStore.update(s => {
        const pods = s.resources[ResourceType.POD].filter(
          p => p.id !== podInfo.name && p.name !== podInfo.name
        );
        
        return {
          ...s,
          resources: {
            ...s.resources,
            [ResourceType.POD]: pods
          }
        };
      });
    });
    
    // Set up event listeners for services
    const serviceUpdatedUnlisten = await listen<any>('k8s:service-updated', (event) => {
      const serviceInfo = event.payload;
      const provider = new GCPProvider();
      const serviceResource = provider.mapToService(serviceInfo);
      
      cloudStore.update(s => {
        const services = s.resources[ResourceType.SERVICE];
        const index = services.findIndex(s => s.id === serviceResource.id || s.name === serviceResource.name);
        
        if (index >= 0) {
          services[index] = serviceResource;
        } else {
          services.push(serviceResource);
        }
        
        return {
          ...s,
          resources: {
            ...s.resources,
            [ResourceType.SERVICE]: [...services]
          }
        };
      });
    });
    
    const serviceDeletedUnlisten = await listen<any>('k8s:service-deleted', (event) => {
      const serviceInfo = event.payload;
      
      cloudStore.update(s => {
        const services = s.resources[ResourceType.SERVICE].filter(
          s => s.id !== serviceInfo.name && s.name !== serviceInfo.name
        );
        
        return {
          ...s,
          resources: {
            ...s.resources,
            [ResourceType.SERVICE]: services
          }
        };
      });
    });
    
    // Set up event listener for watch errors
    let hasStoppedOnError = false; // Guard to prevent multiple stop calls from error handler
    const watchErrorUnlisten = await listen<string>('k8s:watch-error', (event) => {
      const errorMsg = event.payload;
      console.error('Watch error:', errorMsg);
      
      // If it's a connection error, stop watching and let user reconnect
      if (!hasStoppedOnError && (
          errorMsg.includes('tcp connect error') 
          || errorMsg.includes('Cannot assign requested address')
          || errorMsg.includes('error trying to connect')
          || errorMsg.includes('connection refused')
          || errorMsg.includes('connection reset')
      )) {
        hasStoppedOnError = true;
        console.warn('Fatal connection error detected, stopping watches');
        // Use setTimeout to break the reactive cycle
        setTimeout(() => {
          stopWatchingResources().catch(err => {
            console.error('Failed to stop watches after error:', err);
            hasStoppedOnError = false; // Reset on error so we can try again
          });
        }, 0);
      }
    });
    
    // Set up event listeners for deployments
    const deploymentUpdatedUnlisten = await listen<any>('k8s:deployment-updated', (event) => {
      const deploymentInfo = event.payload;
      const provider = new GCPProvider();
      const deploymentResource = provider.mapToDeployment(deploymentInfo);
      
      cloudStore.update(s => {
        const deployments = s.resources[ResourceType.DEPLOYMENT];
        const index = deployments.findIndex(d => d.id === deploymentResource.id || d.name === deploymentResource.name);
        
        if (index >= 0) {
          deployments[index] = deploymentResource;
        } else {
          deployments.push(deploymentResource);
        }
        
        return {
          ...s,
          resources: {
            ...s.resources,
            [ResourceType.DEPLOYMENT]: [...deployments]
          }
        };
      });
    });
    
    const deploymentDeletedUnlisten = await listen<any>('k8s:deployment-deleted', (event) => {
      const deploymentInfo = event.payload;
      
      cloudStore.update(s => {
        const deployments = s.resources[ResourceType.DEPLOYMENT].filter(
          d => d.id !== deploymentInfo.name && d.name !== deploymentInfo.name
        );
        
        return {
          ...s,
          resources: {
            ...s.resources,
            [ResourceType.DEPLOYMENT]: deployments
          }
        };
      });
    });
    
    watchListeners.push(
      podUpdatedUnlisten, 
      podDeletedUnlisten,
      serviceUpdatedUnlisten,
      serviceDeletedUnlisten,
      deploymentUpdatedUnlisten,
      deploymentDeletedUnlisten,
      watchErrorUnlisten
    );
    isWatching = true;
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Unknown error';
    console.error('Failed to start watching resources:', error);
    toastActions.error(`Failed to start watching resources: ${errorMessage}`);
  }
}

/**
 * Stop watching resources
 */
export async function stopWatchingResources(): Promise<void> {
  // Guard against multiple simultaneous calls
  if (isStoppingWatches) {
    return;
  }
  
  isStoppingWatches = true;
  
  try {
    // Stop backend watch tasks
    try {
      await invoke('k8s_stop_all_watches');
    } catch (error) {
      console.error('Failed to stop backend watches:', error);
    }
    
    // Stop frontend listeners
    watchListeners.forEach(unlisten => unlisten());
    watchListeners = [];
    isWatching = false;
  } finally {
    isStoppingWatches = false;
  }
}

