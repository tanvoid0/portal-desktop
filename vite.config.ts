import { defineConfig } from 'vitest/config';
import { playwright } from '@vitest/browser-playwright';
import { sveltekit } from '@sveltejs/kit/vite';
import tailwindcss from '@tailwindcss/vite';
import Icons from 'unplugin-icons/vite';
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { fileURLToPath } from 'url';
import { dirname, resolve } from 'path';

const __dirname = dirname(fileURLToPath(import.meta.url));

const host = process.env.TAURI_DEV_HOST;
const isProduction = process.env.NODE_ENV === 'production';
// A second dev instance (scripts/dev-instance.mjs) needs its own ports so it does
// not collide with, or kill-port, the first. Unset means instance 1: 1420 as before.
const devPort = Number(process.env.PORTAL_DEV_PORT ?? 1420);

/**
 * devicon's @font-face lists eot/ttf/woff/svg. Vite emits an asset for every
 * `url()` it sees, so importing it ships ~9.5 MB of font formats that a Tauri
 * WebView (always Chromium/WebKit) will never request. Rewrite the block to
 * woff only — the relative url stays so the file is still hashed and emitted.
 */
function deviconWoffOnly() {
  return {
    name: 'devicon-woff-only',
    enforce: 'pre' as const,
    transform(code: string, id: string) {
      if (!id.endsWith('.css') || !id.includes('devicon')) return null;
      if (!/@font-face/.test(code)) return null;
      return code.replace(
        /@font-face\s*\{[^}]*\}/,
        `@font-face{font-family:"devicon";src:url("fonts/devicon.woff") format("woff");font-weight:normal;font-style:normal;font-display:block}`
      );
    },
  };
}

export default defineConfig({
  plugins: [
    deviconWoffOnly(),
    tailwindcss(),
    Icons({ compiler: 'svelte' }),
    sveltekit({
      preprocess: vitePreprocess(),
      adapter: adapter({ fallback: 'index.html' }),
      // Kit writes these into its generated tsconfig, which is what gives
      // svelte-check the same `@` / `$lib` resolution Vite has below.
      alias: { '@': 'src', $lib: 'src/lib' },
    }),
  ],

  resolve: {
    alias: {
      '@': resolve(__dirname, 'src'),
      $lib: resolve(__dirname, 'src/lib'),
    },
  },

  // Vite options tailored for Tauri development
  clearScreen: false,
  build: {
    sourcemap: !isProduction,
    minify: isProduction ? 'esbuild' : false,
  },
  server: {
    port: devPort,
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
          port: devPort + 1,
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
          // environment removed (Vitest v4 uses browser.enabled)
          browser: {
            enabled: true,
            provider: playwright(),
            instances: [{ browser: 'chromium' }],
          },
          include: ['src/**/*.svelte.{test,spec}.{js,ts}'],
          exclude: ['src/lib/server/**'],
          setupFiles: ['./vitest-setup-client.ts'],
        },
      },
      {
        extends: './vite.config.ts',
        test: {
          name: 'server',
          environment: 'node',
          include: ['src/**/*.{test,spec}.{js,ts}', 'eslint-rules/*.{test,spec}.mjs'],
          exclude: ['src/**/*.svelte.{test,spec}.{js,ts}'],
        },
      },
    ],
  },
});
