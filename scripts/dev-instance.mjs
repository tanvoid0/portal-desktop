// Run a second (or third, ...) Portal Desktop dev instance alongside the first.
//
//   node scripts/dev-instance.mjs 2
//
// Instance 1 is plain `pnpm tauri:dev` and is left exactly as it was — same vite
// port, same CDP port, same data directory, so existing data is untouched.
// Instance N > 1 gets its own of each:
//
//   vite      1420 + (N-1)   — vite.config.ts reads PORTAL_DEV_PORT
//   CDP       9222 + (N-1)   — WebView2 reads WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
//   data dir  <app data>/instance-N — src-tauri/src/lib.rs reads PORTAL_INSTANCE
//   target    src-tauri/target/instance-N — Windows will not relink a running .exe
//
// The data directory is the one that actually matters: both instances otherwise
// open the same portal_desktop.db, and two SQLite writers on one file corrupts it.
//
// Also serves `predev`, which has to kill the port THIS instance is about to use
// rather than always 1420 — otherwise starting instance 2 kills instance 1.
//
//   node scripts/dev-instance.mjs --kill-port
import { spawn } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { resolve } from 'node:path';

const PORT_BASE = 1420;
const CDP_BASE = 9222;

/** Instance number from argv (launch) or the environment (predev). Defaults to 1. */
function instanceNumber(raw) {
  const n = Number(raw ?? 1);
  if (!Number.isInteger(n) || n < 1 || n > 9) {
    console.error(`dev-instance: expected an instance number 1-9, got ${JSON.stringify(raw)}`);
    process.exit(1);
  }
  return n;
}

/** `tolerateFailure` is for kill-port, which exits non-zero when the port is free. */
function run(command, args, env, { tolerateFailure = false } = {}) {
  // Windows needs a shell to resolve the `pnpm`/`npx` .cmd shims — Node refuses to
  // spawn them directly. Every argument below is therefore shell-safe by
  // construction: a port number, or a relative path with no spaces or quotes.
  const child = spawn(command, args, {
    env,
    stdio: 'inherit',
    shell: process.platform === 'win32',
  });
  child.on('exit', (code) => process.exit(tolerateFailure ? 0 : (code ?? 0)));
  child.on('error', (err) => {
    if (tolerateFailure) process.exit(0);
    console.error(`dev-instance: failed to run ${command}: ${err.message}`);
    process.exit(1);
  });
}

if (process.argv[2] === '--kill-port') {
  // Runs as `predev`, inside the environment `tauri dev` was started with.
  const port = process.env.PORTAL_DEV_PORT ?? String(PORT_BASE);
  run('npx', ['kill-port', port], process.env, { tolerateFailure: true });
} else {
  const n = instanceNumber(process.argv[2]);
  const vitePort = PORT_BASE + (n - 1);
  const cdpPort = CDP_BASE + (n - 1);

  const env = {
    ...process.env,
    PORTAL_DEV_PORT: String(vitePort),
    PORTAL_INSTANCE: String(n),
    WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${cdpPort}`,
  };

  // Instance 1 holds target/debug/portal_desktop.exe open while it runs, and Windows
  // refuses to let cargo replace a running binary ("Access is denied", os error 5), so
  // instance 2 cannot link at all until it builds somewhere else. Costs a separate
  // build tree per instance; there is no sharing a target dir with a running .exe.
  if (n > 1) {
    env.CARGO_TARGET_DIR = resolve('src-tauri/target', `instance-${n}`);
    // Every process sharing a WebView2 user-data folder must pass identical browser
    // arguments. Ours differ by --remote-debugging-port, so instance 2 dies on startup
    // with HRESULT 0x8007139F ("group or resource is not in the correct state") until
    // it gets a folder of its own.
    env.WEBVIEW2_USER_DATA_FOLDER = resolve('src-tauri/target', `instance-${n}`, 'webview2');
  }

  // devUrl lives in tauri.conf.json as a literal; `--config` merges an override.
  // It goes in a file rather than on the command line: the argument would be JSON,
  // and the shell we need on Windows (above) would mangle its quotes. The path is
  // repo-relative so it stays space-free wherever the repo lives.
  const overridePath = `src-tauri/.tauri-instance-${n}.conf.json`;
  writeFileSync(
    overridePath,
    JSON.stringify({ build: { devUrl: `http://localhost:${vitePort}` } }, null, 2)
  );

  console.log(
    `dev-instance ${n}: vite :${vitePort} · CDP :${cdpPort} · data dir ` +
      (n === 1 ? '(default)' : `instance-${n}`)
  );

  run('pnpm', ['exec', 'tauri', 'dev', '--config', overridePath], env);
}
