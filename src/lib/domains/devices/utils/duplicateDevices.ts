import type { DeviceSummary } from '../types';

// No stable device fingerprint exists in the pairing protocol (device_id is a
// fresh random UUID per request) — device_name is the only signal available
// to tell "same phone reconnecting" from "a different device entirely".
export function normalizeDeviceName(name: string): string {
  return name.trim().toLowerCase();
}

export function sameCapabilities(a: string[], b: string[]): boolean {
  if (a.length !== b.length) return false;
  const sorted = [...b].sort();
  return [...a].sort().every((cap, i) => cap === sorted[i]);
}

/** Other currently-live devices sharing `device.device_name`, excluding `device` itself. */
export function findDuplicatesByName(
  device: DeviceSummary,
  liveDevices: DeviceSummary[]
): DeviceSummary[] {
  const name = normalizeDeviceName(device.device_name);
  return liveDevices.filter(
    (d) => d.device_id !== device.device_id && normalizeDeviceName(d.device_name) === name
  );
}
