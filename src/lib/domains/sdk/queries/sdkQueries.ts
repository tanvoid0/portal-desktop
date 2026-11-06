import { queryKeys } from '$lib/domains/shared/query/keys';
import { sdkConfigService } from '../services/sdkConfigService';
import { listSDKManagers } from '../services/sdkManagerService';

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
