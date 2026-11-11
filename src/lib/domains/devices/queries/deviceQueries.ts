import { createQuery } from '@tanstack/svelte-query';
import { queryKeys } from '$lib/domains/shared/query/keys';
import {
  fetchConnectivityStatus,
  fetchDevices,
  fetchLanEnabled,
  fetchRemoteCapabilities,
} from '../api/deviceApi';

export function createDevicesQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.devices.all,
    queryFn: fetchDevices,
  }));
}

/** Capability names never change at runtime — they're a fixed registry. */
export function createCapabilitiesQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.devices.capabilities,
    queryFn: fetchRemoteCapabilities,
    staleTime: Infinity,
  }));
}

export function createLanEnabledQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.devices.lanEnabled,
    queryFn: fetchLanEnabled,
  }));
}

export function createConnectivityStatusQuery() {
  return createQuery(() => ({
    queryKey: queryKeys.devices.connectivity,
    queryFn: fetchConnectivityStatus,
  }));
}
