import { invokeClient } from '$lib/utils/invokeClient';
import type {
  ApprovalType,
  ApproveDeviceResponse,
  DeviceSummary,
  PairingQrResponse,
} from '../types';

export function fetchDevices(): Promise<DeviceSummary[]> {
  return invokeClient.post<DeviceSummary[]>('list_devices');
}

export function fetchRemoteCapabilities(): Promise<string[]> {
  return invokeClient.post<string[]>('remote_list_capabilities');
}

export function fetchLanEnabled(): Promise<boolean> {
  return invokeClient.post<boolean>('get_remote_lan_enabled');
}

export function setLanEnabled(enabled: boolean): Promise<void> {
  return invokeClient.post('set_remote_lan_enabled', { enabled });
}

export function approveDevice(
  deviceId: string,
  approvalType: ApprovalType,
  capabilities: string[]
): Promise<ApproveDeviceResponse> {
  return invokeClient.post<ApproveDeviceResponse>('approve_device', {
    request: { device_id: deviceId, approval_type: approvalType, capabilities },
  });
}

export function rejectDevice(deviceId: string): Promise<void> {
  return invokeClient.post('reject_device', { deviceId });
}

export function revokeDevice(deviceId: string): Promise<void> {
  return invokeClient.post('revoke_device', { deviceId });
}

export function updateDeviceCapabilities(deviceId: string, capabilities: string[]): Promise<void> {
  return invokeClient.post('update_device_capabilities', { deviceId, capabilities });
}

export function generatePairingQr(capabilities: string[]): Promise<PairingQrResponse> {
  return invokeClient.post<PairingQrResponse>('generate_pairing_qr', { capabilities });
}
