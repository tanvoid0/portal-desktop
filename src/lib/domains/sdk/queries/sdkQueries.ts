import type { CreateQueryResult } from '@tanstack/svelte-query';
import { queryKeys } from '$lib/domains/shared/query/keys';
import { sdkConfigService, type ProcessedSDKConfig } from '../services/sdkConfigService';
import { listSDKManagers, type SDKManagerInfo } from '../services/sdkManagerService';

// Shared by the sidebar and every /sdk page so one probe pass serves them all.
export const sdkConfigsQuery = () => ({
  queryKey: queryKeys.sdk.configs,
  queryFn: () => sdkConfigService.getAllSDKConfigs(),
  refetchOnWindowFocus: false, // the probe pass takes ~10s
});

export const sdkManagersQuery = () => ({
  queryKey: queryKeys.sdk.managers,
  queryFn: () => listSDKManagers(),
  refetchOnWindowFocus: false,
});

export interface SdkQueries {
  configsQuery: CreateQueryResult<ProcessedSDKConfig[]>;
  managersQuery: CreateQueryResult<SDKManagerInfo[]>;
}

/**
 * `/sdk/+layout.svelte` creates the two queries above exactly once and sets this context —
 * the sidebar and every `/sdk/*` page read them from here instead of each calling
 * `createQuery` again. On a cold mount, two independent `createQuery` calls for the identical
 * key could end up as two separate observers racing each other (one resolving, the other
 * stuck); sharing a single pair of observers makes that divergence structurally impossible.
 */
export const SDK_QUERIES_CONTEXT = Symbol('sdk-queries');
