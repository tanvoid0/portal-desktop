import { defineConfig } from "vitest/config";
import { playwright } from "@vitest/browser-playwright";
import { sveltekit } from "@sveltejs/kit/vite";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;
const isProduction = process.env.NODE_ENV === "production";

/**
 * devicon's @font-face lists eot/ttf/woff/svg. Vite emits an asset for every
 * `url()` it sees, so importing it ships ~9.5 MB of font formats that a Tauri
 * WebView (always Chromium/WebKit) will never request. Rewrite the block to
 * woff only — the relative url stays so the file is still hashed and emitted.
 */
function deviconWoffOnly() {
  return {
    name: "devicon-woff-only",
    enforce: "pre" as const,
    transform(code: string, id: string) {
      if (!id.endsWith(".css") || !id.includes("devicon")) return null;
      if (!/@font-face/.test(code)) return null;
      return code.replace(
        /@font-face\s*\{[^}]*\}/,
        `@font-face{font-family:"devicon";src:url("fonts/devicon.woff") format("woff");font-weight:normal;font-style:normal;font-display:block}`,
      );
    },
  };
}

export default defineConfig({
  plugins: [deviconWoffOnly(), tailwindcss(), sveltekit()],

  // Vite options tailored for Tauri development
  clearScreen: false,
  build: {
    sourcemap: !isProduction,
    minify: isProduction ? "esbuild" : false,
  },
  server: {
    port: 1420,
    strictPort: true,
    // Allow network access for QR code sharing feature
    // Set TAURI_DEV_HOST environment variable to your local IP for network access
    // Example: TAURI_DEV_HOST=192.168.1.100 pnpm dev
    // Or set to '0.0.0.0' to allow access from any network interface
    host: host || "0.0.0.0",
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Tell Vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
  test: {
    expect: { requireAssertions: true },
    projects: [
      {
        extends: "./vite.config.ts",
        test: {
          name: "client",
          // environment removed (Vitest v4 uses browser.enabled)
          browser: {
            enabled: true,
            provider: playwright,
            instances: [{ browser: "chromium" }],
          },
          include: ["src/**/*.svelte.{test,spec}.{js,ts}"],
          exclude: ["src/lib/server/**"],
          setupFiles: ["./vitest-setup-client.ts"],
        },
      },
      {
        extends: "./vite.config.ts",
        test: {
          name: "server",
          environment: "node",
          include: ["src/**/*.{test,spec}.{js,ts}"],
          exclude: ["src/**/*.svelte.{test,spec}.{js,ts}"],
        },
      },
    ],
  },
});
