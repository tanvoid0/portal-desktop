import { isTauriEnvironment, tauriInvoke } from '$lib/utils/tauri';

const DEFAULT_PORT = 1420;

/**
 * Network service for detecting local network IP address
 */
export class NetworkService {
	/**
	 * Get the local network IP address
	 * Tries Tauri command first, falls back to JavaScript detection
	 */
	static async getLocalNetworkIP(): Promise<string> {
		if (isTauriEnvironment()) {
			try {
				const ip = await tauriInvoke<string>('get_local_network_ip');
				return ip;
			} catch (error) {
				console.warn('Failed to get IP from Tauri, falling back to JavaScript method:', error);
				return this.getLocalIPFallback();
			}
		} else {
			return this.getLocalIPFallback();
		}
	}

	/**
	 * Fallback method using JavaScript WebRTC
	 * This works in browsers but is less reliable
	 */
	private static getLocalIPFallback(): Promise<string> {
		return new Promise((resolve) => {
			// Try WebRTC method
			const RTCPeerConnection = window.RTCPeerConnection || 
				(window as any).webkitRTCPeerConnection || 
				(window as any).mozRTCPeerConnection;

			if (!RTCPeerConnection) {
				// Fallback to localhost
				resolve('127.0.0.1');
				return;
			}

			const pc = new RTCPeerConnection({
				iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
			});

			const ips: string[] = [];
			pc.createDataChannel('');

			pc.onicecandidate = (event) => {
				if (event.candidate) {
					const candidate = event.candidate.candidate;
					const match = candidate.match(/([0-9]{1,3}(\.[0-9]{1,3}){3})/);
					if (match) {
						const ip = match[1];
						// Filter out loopback and link-local
						if (ip !== '127.0.0.1' && !ip.startsWith('169.254.')) {
							ips.push(ip);
						}
					}
				} else {
					// All candidates gathered
					pc.close();
					if (ips.length > 0) {
						// Prefer private IPs
						const privateIP = ips.find(ip => 
							ip.startsWith('192.168.') || 
							ip.startsWith('10.') || 
							ip.startsWith('172.16.') || 
							ip.startsWith('172.17.') || 
							ip.startsWith('172.18.') || 
							ip.startsWith('172.19.') || 
							ip.startsWith('172.20.') || 
							ip.startsWith('172.21.') || 
							ip.startsWith('172.22.') || 
							ip.startsWith('172.23.') || 
							ip.startsWith('172.24.') || 
							ip.startsWith('172.25.') || 
							ip.startsWith('172.26.') || 
							ip.startsWith('172.27.') || 
							ip.startsWith('172.28.') || 
							ip.startsWith('172.29.') || 
							ip.startsWith('172.30.') || 
							ip.startsWith('172.31.')
						);
						resolve(privateIP || ips[0] || '127.0.0.1');
					} else {
						resolve('127.0.0.1');
					}
				}
			};

			pc.createOffer()
				.then(offer => pc.setLocalDescription(offer))
				.catch(() => {
					pc.close();
					resolve('127.0.0.1');
				});

			// Timeout after 3 seconds
			setTimeout(() => {
				pc.close();
				if (ips.length > 0) {
					resolve(ips[0]);
				} else {
					resolve('127.0.0.1');
				}
			}, 3000);
		});
	}

	/**
	 * Get the full URL for accessing the application
	 */
	static async getApplicationURL(port: number = DEFAULT_PORT): Promise<string> {
		const ip = await this.getLocalNetworkIP();
		return `http://${ip}:${port}`;
	}

	/**
	 * Get the current port (from environment or default)
	 */
	static getPort(): number {
		// Try to get from window location if available
		if (typeof window !== 'undefined' && window.location.port) {
			const port = parseInt(window.location.port, 10);
			if (!isNaN(port)) {
				return port;
			}
		}
		return DEFAULT_PORT;
	}
}

