/**
 * Domain Registry - Centralized configuration and validation
 * 
 * This file provides centralized access to all domains and their exports.
 * It follows the domain-driven design (DDD) approach with clean imports.
 * 
 * Import Pattern: Always use `$lib/domains/domainName` instead of relative `../../` paths
 */

// Domain Registry Configuration
export const DOMAIN_REGISTRY = {
  projects: {
    name: 'projects',
    path: 'projects',
    description: 'Project management domain',
    dependencies: ['shared']
  },
  deployments: {
    name: 'deployments', 
    path: 'deployments',
    description: 'Deployment automation domain',
    dependencies: ['shared', 'projects']
  },
  credentials: {
    name: 'credentials',
    path: 'credentials', 
    description: 'Credential management domain',
    dependencies: ['shared']
  },
  settings: {
    name: 'settings',
    path: 'settings',
    description: 'Application settings domain', 
    dependencies: ['shared']
  },
  terminal: {
    name: 'terminal',
    path: 'terminal',
    description: 'Terminal session domain',
    dependencies: ['shared']
  },
  notifications: {
    name: 'notifications',
    path: 'notifications',
    description: 'Notification system domain',
    dependencies: ['shared']
  },
  performance: {
    name: 'performance',
    path: 'performance', 
    description: 'Performance monitoring domain',
    dependencies: ['shared']
  },
  shared: {
    name: 'shared',
    path: 'shared',
    description: 'Shared utilities and services',
    dependencies: []
  }
} as const;

// Domain validation
export function validateDomain(domainName: string): boolean {
  return domainName in DOMAIN_REGISTRY;
}

export function getDomainInfo(domainName: string) {
  if (!validateDomain(domainName)) {
    throw new Error(`Invalid domain: ${domainName}`);
  }
  return DOMAIN_REGISTRY[domainName as keyof typeof DOMAIN_REGISTRY];
}

// Re-export all domain modules for clean imports
// This allows importing from $lib/domains/domainName

// Shared domain exports
export * from './shared';

// Project domain exports  
export * from './projects';

// Deployment domain exports
export * from './deployments';

// Credentials domain exports
export * from './credentials';

// Settings domain exports
export * from './settings';

// Terminal domain exports
export * from './terminal';

// Notifications domain exports (stub - not yet implemented)
// export * from './notifications';

// Performance domain exports (stub - not yet implemented)
// export * from './performance';
