import { queryKeys } from '$lib/domains/shared/query/keys';
import { sdkConfigService } from '../services/sdkConfigService';
import { listSDKManagers } from '../services/sdkManagerService';

// Each consumer calls `createQuery(sdkConfigsQuery)` itself; TanStack dedupes by key, so one
// probe pass still serves the sidebar and every /sdk page. Sharing one result object through
// context let a render stall in the page freeze the sidebar too, so don't go back to that.
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
