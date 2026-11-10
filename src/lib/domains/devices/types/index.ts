export type { DeviceSummary } from '../generated';

/** `approve_device`'s three grant lifetimes. */
export type ApprovalType = 'temporary' | 'long_term' | 'until_revoked';

/** `approve_device`'s response — untyped on the wire (request-layer struct, no ts-rs). */
export interface ApproveDeviceResponse {
  success: boolean;
  expires_at: string | null;
  message: string;
}

/** `generate_pairing_qr`'s response — untyped on the wire (request-layer struct, no ts-rs). */
export interface PairingQrResponse {
  qr_url: string;
  device_id: string;
}
