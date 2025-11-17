// Base provider implementation with common functionality
// All provider implementations should extend this class

import type {
  ICloudProvider,
  ICluster,
  ICloudResource,
  ResourceEvent
} from '../../core/types';
import {
  ResourceType,
  ClusterStatus,
  CloudProviderType,
  ResourceStatus
} from '../../core/types';

export abstract class BaseProvider implements ICloudProvider {
  abstract readonly name: string;
  abstract readonly type: CloudProviderType;
  
  protected connected = false;
  protected currentCluster: ICluster | null = null;
  
  // Abstract methods that must be implemented by subclasses
  abstract initialize(): Promise<void>;
  abstract connect(clusterId: string): Promise<void>;
  abstract disconnect(): Promise<void>;
  
  abstract listClusters(): Promise<ICluster[]>;
  abstract getCluster(id: string): Promise<ICluster | null>;
  abstract listResources(type: ResourceType, namespace?: string): Promise<ICloudResource[]>;
  abstract getResource(type: ResourceType, id: string, namespace: string): Promise<ICloudResource | null>;
  abstract watchResources(type: ResourceType, namespace?: string): AsyncIterable<ResourceEvent>;
  abstract listNamespaces(): Promise<string[]>;
  
  // Common implementations
  
  async isConnected(): Promise<boolean> {
    return this.connected;
  }
  
  async getCurrentCluster(): Promise<ICluster | null> {
    return this.currentCluster;
  }
  
  // Common utility methods
  protected validateCluster(cluster: ICluster): boolean {
    return cluster.status === ClusterStatus.CONNECTED || cluster.status === ClusterStatus.READY;
  }
  
  protected mapResourceStatus(status: string): ResourceStatus {
    const statusLower = status.toLowerCase();
    switch (statusLower) {
      case 'running':
        return ResourceStatus.RUNNING;
      case 'pending':
        return ResourceStatus.PENDING;
      case 'failed':
      case 'error':
      case 'crashloopbackoff':
        return ResourceStatus.FAILED;
      case 'succeeded':
        return ResourceStatus.SUCCEEDED;
      case 'terminating':
        return ResourceStatus.TERMINATING;
      default:
        return ResourceStatus.UNKNOWN;
    }
  }
  
  protected mapClusterStatus(status: string | ClusterStatus): ClusterStatus {
    if (typeof status === 'string') {
      const statusLower = status.toLowerCase();
      switch (statusLower) {
        case 'connected':
          return ClusterStatus.CONNECTED;
        case 'disconnected':
          return ClusterStatus.DISCONNECTED;
        case 'connecting':
          return ClusterStatus.CONNECTING;
        case 'ready':
          return ClusterStatus.READY;
        case 'error':
          return ClusterStatus.ERROR;
        default:
          return ClusterStatus.DISCONNECTED;
      }
    }
    return status;
  }
}

