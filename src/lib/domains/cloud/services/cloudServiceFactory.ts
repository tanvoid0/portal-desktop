// Service factory for creating and managing cloud provider instances
// Uses singleton pattern to ensure only one instance per provider type

import type { ICloudProvider } from '../core/types';
import { CloudProviderType } from '../core/types';
import { BaseProvider } from '../providers/base/BaseProvider';

// Lazy import providers to avoid circular dependencies
let GCPProvider: typeof import('../providers/gcp/GCPProvider').GCPProvider;

export class CloudServiceFactory {
  private static providers = new Map<CloudProviderType, ICloudProvider>();
  
  /**
   * Get or create a provider instance for the given type
   */
  static async getProvider(type: CloudProviderType): Promise<ICloudProvider> {
    if (!this.providers.has(type)) {
      const provider = await this.createProvider(type);
      await provider.initialize();
      this.providers.set(type, provider);
    }
    return this.providers.get(type)!;
  }
  
  /**
   * Create a new provider instance (does not initialize)
   */
  private static async createProvider(type: CloudProviderType): Promise<ICloudProvider> {
    switch (type) {
      case CloudProviderType.GCP:
        // Lazy load GCP provider
        if (!GCPProvider) {
          const module = await import('../providers/gcp/GCPProvider');
          GCPProvider = module.GCPProvider;
        }
        return new GCPProvider();
        
      case CloudProviderType.AWS:
        throw new Error('AWS provider not yet implemented');
        
      case CloudProviderType.AZURE:
        throw new Error('Azure provider not yet implemented');
        
      case CloudProviderType.DIGITAL_OCEAN:
        throw new Error('Digital Ocean provider not yet implemented');
        
      default:
        throw new Error(`Unknown provider type: ${type}`);
    }
  }
  
  /**
   * Remove a provider instance from the cache
   */
  static removeProvider(type: CloudProviderType): void {
    const provider = this.providers.get(type);
    if (provider) {
      provider.disconnect().catch(console.error);
      this.providers.delete(type);
    }
  }
  
  /**
   * Get all available provider types
   */
  static getAvailableProviders(): CloudProviderType[] {
    return [
      CloudProviderType.GCP,
      // Future: CloudProviderType.AWS,
      // Future: CloudProviderType.AZURE,
      // Future: CloudProviderType.DIGITAL_OCEAN,
    ];
  }
  
  /**
   * Clear all provider instances
   */
  static clearAll(): void {
    for (const [type] of this.providers) {
      this.removeProvider(type);
    }
  }
}

