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
//   daemon    18421 + (N-1)  — daemon::configured_port reads PORTAL_DAEMON_PORT
//   target    I:/build/cargo/portal-desktop-instance-N — Windows will not relink a
//             running .exe. Per-machine: CARGO_TARGET_DIR / CARGO_BUILD_BUILD_DIR
//             override it, and with no I: drive it is src-tauri/target/instance-N.
//
// The data directory is the one that actually matters: both instances otherwise
// open the same portal_desktop.db, and two SQLite writers on one file corrupts it.
//
// Also serves `predev`, which has to kill the port THIS instance is about to use
// rather than always 1420 — otherwise starting instance 2 kills instance 1.
//
//   node scripts/dev-instance.mjs --kill-port
import { execFileSync, spawn } from 'node:child_process';
import { existsSync, writeFileSync } from 'node:fs';
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

function run(command, args, env) {
  // Windows needs a shell to resolve the `pnpm` .cmd shim — Node refuses to
  // spawn it directly. Every argument below is therefore shell-safe by
  // construction: a port number, or a relative path with no spaces or quotes.
  const child = spawn(command, args, {
    env,
    stdio: 'inherit',
    shell: process.platform === 'win32',
  });
  child.on('exit', (code) => process.exit(code ?? 0));
  child.on('error', (err) => {
    console.error(`dev-instance: failed to run ${command}: ${err.message}`);
    process.exit(1);
  });
}

/** Pids `LISTENING` on `port` — Windows via `netstat -ano`, else `lsof -t`. Same
 * approach as `usage::owning_pids` in src-tauri (the port-kill logic the app
 * itself uses), so this script doesn't invent a second way to find a port's owner. */
function owningPids(port) {
  try {
    if (process.platform === 'win32') {
      // Not the State column — it's localized ("LISTENING", "ABHÖREN", ...;
      // #393/#719). Same check as usage::parse_listening_pids: a listening TCP
      // row has 5 fields and the unspecified Foreign Address, an IP literal.
      return [
        ...new Set(
          execFileSync('netstat', ['-ano'], { encoding: 'utf8' })
            .split('\n')
            .map((l) => l.trim().split(/\s+/))
            .filter(
              (f) =>
                f.length === 5 &&
                (f[2] === '0.0.0.0:0' || f[2] === '[::]:0') &&
                f[1].endsWith(`:${port}`)
            )
            .map((f) => f[4])
        ),
      ];
    }
    return execFileSync('lsof', ['-nP', `-iTCP:${port}`, '-sTCP:LISTEN', '-t'], {
      encoding: 'utf8',
    })
      .split('\n')
      .map((l) => l.trim())
      .filter(Boolean);
  } catch {
    // lsof/netstat exits non-zero when nothing matches — same as "no owner".
    return [];
  }
}

/** Kills whatever's listening on `port`, via `taskkill /T /F` (Windows) or
 * `kill -9` (unix) per pid — mirrors `move_commands::kill_command`. Best-effort:
 * a pid that's already gone by the time we get to it is not an error. */
function killPort(port) {
  const pids = owningPids(port);
  for (const pid of pids) {
    try {
      if (process.platform === 'win32') execFileSync('taskkill', ['/PID', pid, '/T', '/F']);
      else execFileSync('kill', ['-9', pid]);
    } catch {
      // already exited, or we lack permission — `tauri dev` fails loudly on
      // the port right after if it's genuinely still held.
    }
  }
  console.log(
    pids.length
      ? `dev-instance: killed ${pids.length} process(es) on port ${port}`
      : `dev-instance: port ${port} was already free`
  );
}

if (process.argv[2] === '--kill-port') {
  // Runs as `predev`, inside the environment `tauri dev` was started with.
  // 18421 is a debug build's daemon (daemon::DEV_PORT); 18420 is the installed
  // release, which has no vite port to protect.
  const daemonPort = process.env.PORTAL_DAEMON_PORT ?? '18421';
  const alreadyUp = await fetch(`http://127.0.0.1:${daemonPort}/health`, {
    signal: AbortSignal.timeout(1500),
  })
    .then((r) => r.ok)
    .catch(() => false);
  if (alreadyUp) {
    // A live instance is already running — killing the vite port would sever
    // its frontend out from under it. Let `tauri dev` fail loudly on the port
    // instead of silently wrecking the other instance.
    console.log('dev-instance: Portal is already running — skipping kill-port');
  } else {
    const port = process.env.PORTAL_DEV_PORT ?? String(PORT_BASE);
    killPort(port);
  }
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
    // Instance 1's daemon holds 18421 (daemon::DEV_PORT).
    env.PORTAL_DAEMON_PORT = String(18421 + (n - 1));
    // I: has the free space (D: runs tight; a cold build is 3+ GB). A caller-set
    // CARGO_TARGET_DIR / CARGO_BUILD_BUILD_DIR wins; no I: drive falls back to D:.
    env.CARGO_TARGET_DIR ||= existsSync('I:/')
      ? `I:/build/cargo/portal-desktop-instance-${n}`
      : resolve('src-tauri/target', `instance-${n}`);
    // The config's build-dir is per workspace path, so without this instance 2 would
    // relink into instance 1's deps/, where its running .exe is hard-linked from.
    env.CARGO_BUILD_BUILD_DIR ||= env.CARGO_TARGET_DIR;
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
