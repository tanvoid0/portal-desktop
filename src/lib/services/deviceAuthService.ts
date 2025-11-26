/**
 * Device Authentication Service
 * 
 * Handles device authentication flow for browser access:
 * 1. Generate device ID
 * 2. Request passcode from host
 * 3. Verify passcode
 * 4. Store access token
 * 5. Use token for API requests
 */

export interface DeviceInfo {
	device_id: string;
	device_name: string;
	device_info: {
		userAgent: string;
		platform: string;
		ip?: string;
	};
}

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
	 * Get device name (user-friendly)
	 */
	static getDeviceName(): string {
		if (typeof window === 'undefined') {
			return 'Unknown Device';
		}

		const ua = navigator.userAgent;
		if (ua.includes('Mobile')) {
			return 'Mobile Device';
		} else if (ua.includes('Tablet')) {
			return 'Tablet';
		} else {
			return 'Browser';
		}
	}

	/**
	 * Get device info
	 */
	static getDeviceInfo(): DeviceInfo {
		return {
			device_id: this.getDeviceId(),
			device_name: this.getDeviceName(),
			device_info: {
				userAgent: typeof navigator !== 'undefined' ? navigator.userAgent : 'Unknown',
				platform: typeof navigator !== 'undefined' ? navigator.platform : 'Unknown',
			},
		};
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
	 * Save auth state
	 */
	static saveAuthState(state: Partial<AuthState>): void {
		if (typeof window === 'undefined') return;

		const current = this.getAuthState();
		const updated = { ...current, ...state };
		localStorage.setItem(this.STORAGE_KEY, JSON.stringify(updated));
	}

	/**
	 * Clear auth state
	 */
	static clearAuthState(): void {
		if (typeof window === 'undefined') return;
		localStorage.removeItem(this.STORAGE_KEY);
	}

	/**
	 * Request passcode from host
	 */
	static async requestPasscode(): Promise<string> {
		const deviceInfo = this.getDeviceInfo();
		
		// Use InvokeClient with requireAuth: false since this is a public command
		const { invokeClient } = await import('$lib/utils/invokeClient');
		
		const result = await invokeClient.post<{ passcode: string }>('generate_device_passcode', {
			device_id: deviceInfo.device_id,
			device_name: deviceInfo.device_name,
			device_info: JSON.stringify(deviceInfo.device_info),
		}, {
			requireAuth: false,
		});
		
		return result.passcode;
	}

	/**
	 * Verify passcode and get access token
	 */
	static async verifyPasscode(passcode: string, approvalType: 'temporary' | 'long_term' = 'temporary'): Promise<void> {
		const deviceInfo = this.getDeviceInfo();

		// Use InvokeClient with requireAuth: false since this is a public command
		const { invokeClient } = await import('$lib/utils/invokeClient');
		
		// First verify the passcode
		const verifyResult = await invokeClient.post<{
			approved: boolean;
			access_token: string | null;
			expires_at: string | null;
			message: string;
		}>('verify_device_passcode', {
			device_id: deviceInfo.device_id,
			passcode,
			approval_type: approvalType,
		}, {
			requireAuth: false,
		});
		
		// Passcode is verified, now wait for host approval
		// The host will approve via the DeviceApprovalDialog
		// We'll poll for the access token
		await this.pollForApproval(deviceInfo.device_id, approvalType);
	}

	/**
	 * Poll for device approval
	 * Checks if device has been approved and access token is available
	 * The host device will call approve_device, which generates the access token
	 * We poll by checking device status endpoint
	 */
	private static async pollForApproval(deviceId: string, approvalType: string, maxAttempts: number = 60): Promise<void> {
		// Poll every 2 seconds for up to 2 minutes
		const { invokeClient } = await import('$lib/utils/invokeClient');
		
		for (let i = 0; i < maxAttempts; i++) {
			await new Promise(resolve => setTimeout(resolve, 2000));

			try {
				// Check device status to see if approved (public command, no auth required)
				const status = await invokeClient.post<{
					approved: boolean;
					access_token: string | null;
					expires_at: string | null;
					message: string;
				}>('get_device_status', {
					device_id: deviceId,
				}, {
					requireAuth: false,
				});

				if (status.approved && status.access_token) {
					// Device approved! Save the token
					this.saveAuthState({
						access_token: status.access_token,
						token_expires_at: status.expires_at,
						is_authenticated: true,
					});
					return;
				}
			} catch (error) {
				console.error('Error polling for approval:', error);
			}
		}

		throw new Error('Device approval timeout. The host device may not have approved your request. Please request a new passcode.');
	}

	/**
	 * Get access token for API requests
	 */
	static getAccessToken(): string | null {
		const state = this.getAuthState();
		return state.access_token;
	}

	/**
	 * Check if device is authenticated
	 */
	static isAuthenticated(): boolean {
		return this.getAuthState().is_authenticated;
	}
}

export default DeviceAuthService;

