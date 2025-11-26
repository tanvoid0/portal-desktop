import { defineConfig } from 'vitest/config';
import { sveltekit } from '@sveltejs/kit/vite';

const host = process.env.TAURI_DEV_HOST;
const isProduction = process.env.NODE_ENV === 'production';

export default defineConfig({
	plugins: [
		sveltekit(),
		// Strip console.log and console.debug in production
		isProduction && {
			name: 'strip-console',
			transform(code: string, id: string) {
				if (id.includes('node_modules')) return null;
				return {
					code: code
						.replace(/console\.(log|debug)\([^)]*\);?/g, '')
						.replace(/console\.(log|debug)\([^)]*\)/g, ''),
					map: null
				};
			}
		}
	].filter(Boolean),
	
	// Vite options tailored for Tauri development
	clearScreen: false,
	build: {
		sourcemap: !isProduction,
		minify: isProduction ? 'esbuild' : false,
	},
	server: {
		port: 1420,
		strictPort: true,
		// Allow network access for QR code sharing feature
		// Set TAURI_DEV_HOST environment variable to your local IP for network access
		// Example: TAURI_DEV_HOST=192.168.1.100 pnpm dev
		// Or set to '0.0.0.0' to allow access from any network interface
		host: host || '0.0.0.0',
		hmr: host
			? {
					protocol: 'ws',
					host,
					port: 1421,
				}
			: undefined,
		watch: {
			// Tell Vite to ignore watching `src-tauri`
			ignored: ['**/src-tauri/**'],
		},
	},
	test: {
		expect: { requireAssertions: true },
		projects: [
			{
				extends: './vite.config.ts',
				test: {
					name: 'client',
					environment: 'browser',
					browser: {
						enabled: true,
						provider: 'playwright',
						instances: [{ browser: 'chromium' }]
					},
					include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
					exclude: ['src/lib/server/**'],
					setupFiles: ['./vitest-setup-client.ts']
				}
			},
			{
				extends: './vite.config.ts',
				test: {
					name: 'server',
					environment: 'node',
					include: ['src/**/*.{test,spec}.{js,ts}'],
					exclude: ['src/**/*.svelte.{test,spec}.{js,ts}']
				}
			}
		]
	}
});
