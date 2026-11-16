/**
 * Device Authentication Service
 *
 * Remote-browser passcode auth (request/verify passcode, poll for approval) is gone along
 * with the backend commands it called — see DeviceAuthGuard.svelte for why. What's left is
 * just the access-token storage `invokeClient.ts` still reads for the (currently unreachable,
 * see its own comments) remote-browser HTTP branch.
 */

export interface AuthState {
  device_id: string | null;
  access_token: string | null;
  token_expires_at: string | null;
  is_authenticated: boolean;
}

class DeviceAuthService {
  private static readonly STORAGE_KEY = 'device_auth_state';
  private static readonly DEVICE_ID_KEY = 'device_id';

  /**
   * Generate or retrieve device ID
   */
  static getDeviceId(): string {
    if (typeof window === 'undefined') {
      return 'unknown';
    }

    let deviceId = localStorage.getItem(this.DEVICE_ID_KEY);
    if (!deviceId) {
      // Generate a unique device ID
      deviceId = `device_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
      localStorage.setItem(this.DEVICE_ID_KEY, deviceId);
    }
    return deviceId;
  }

  /**
   * Get current auth state
   */
  static getAuthState(): AuthState {
    if (typeof window === 'undefined') {
      return {
        device_id: null,
        access_token: null,
        token_expires_at: null,
        is_authenticated: false,
      };
    }

    const stored = localStorage.getItem(this.STORAGE_KEY);
    if (!stored) {
      return {
        device_id: this.getDeviceId(),
        access_token: null,
        token_expires_at: null,
        is_authenticated: false,
      };
    }

    try {
      const state = JSON.parse(stored);
      // Check if token is expired
      if (state.token_expires_at) {
        const expiresAt = new Date(state.token_expires_at);
        if (expiresAt < new Date()) {
          // Token expired, clear state
          this.clearAuthState();
          return {
            device_id: this.getDeviceId(),
            access_token: null,
            token_expires_at: null,
            is_authenticated: false,
          };
        }
      }
      return {
        ...state,
        is_authenticated: !!state.access_token,
      };
    } catch {
      return {
        device_id: this.getDeviceId(),
        access_token: null,
        token_expires_at: null,
        is_authenticated: false,
      };
    }
  }

  /**
   * Clear auth state
   */
  static clearAuthState(): void {
    if (typeof window === 'undefined') return;
    localStorage.removeItem(this.STORAGE_KEY);
  }

  /**
   * Get access token for API requests
   */
  static getAccessToken(): string | null {
    const state = this.getAuthState();
    return state.access_token;
  }
}

export default DeviceAuthService;
