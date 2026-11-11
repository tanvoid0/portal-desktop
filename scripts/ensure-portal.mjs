// What any Claude session/agent should run instead of blindly calling
// `pnpm tauri:dev` when it just wants Portal's shared daemon (port 18420) up
// and reachable — see this repo's CLAUDE.md. Idempotent and lock-protected:
// safe to call from multiple sessions/agents at once.
//
//   node scripts/ensure-portal.mjs
//
// Builds and runs the plain Rust binary directly — `cargo build` then launch
// the exe — rather than going through `tauri dev`/vite. This is a background
// service launch, not an interactive dev session: nobody here wants hot
// reload, and routing through the tauri-cli -> pnpm -> vite chain turned out
// to swallow the child's stdout/stderr entirely when detached on Windows
// (the log stayed empty however it was piped), with no error to explain why.
// A plain `cargo build` + direct exe launch has none of that: both report to
// stdout the normal way, which is all we need to pipe to a log file.
// The webview's content (devUrl vs bundled frontendDist) is irrelevant here —
// the HTTP daemon binds during `setup()` regardless of whether the window
// has anything to show, and the daemon is the only thing any of this cares
// about.
//
// ponytail: a lock file + PID liveness check is a simpler guard than a real
// mutex/IPC mechanism — good enough for "don't launch two of these on one
// machine". Upgrade to something more robust only if this races in practice.
import { spawn } from 'node:child_process';
import { createWriteStream, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { homedir, tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const REPO_ROOT = dirname(dirname(fileURLToPath(import.meta.url)));
const SRC_TAURI = join(REPO_ROOT, 'src-tauri');
const EXE_PATH = join(
  SRC_TAURI,
  'target',
  'debug',
  process.platform === 'win32' ? 'portal_desktop.exe' : 'portal_desktop'
);

const PORT = process.env.PORTAL_DAEMON_PORT ?? '18420';
const HEALTH_URL = `http://127.0.0.1:${PORT}/health`;
const LOCK_DIR = join(homedir(), '.portal-desktop');
const LOCK_PATH = join(LOCK_DIR, 'daemon.lock');
const LOG_PATH = join(tmpdir(), 'portal-desktop-ensure.log');

async function health(timeoutMs = 1500) {
  try {
    const res = await fetch(HEALTH_URL, { signal: AbortSignal.timeout(timeoutMs) });
    return res.ok;
  } catch {
    return false;
  }
}

async function pollHealth(maxMs) {
  const deadline = Date.now() + maxMs;
  while (Date.now() < deadline) {
    if (await health()) return true;
    await new Promise((r) => setTimeout(r, 2000));
  }
  return false;
}

function pidAlive(pid) {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

/** Last `n` lines of the spawn log, for printing inline on failure — the
 * caller (another session, an agent) should not have to go find the file
 * itself just to learn why the build failed. */
function logTail(n = 40) {
  try {
    const lines = readFileSync(LOG_PATH, 'utf8').split('\n');
    return lines.slice(-n).join('\n').trim();
  } catch {
    return '';
  }
}

// Exit codes below release the lock first — `process.exit()` tears the
// process down immediately and does not run a pending `finally`, so cleanup
// has to happen before each call, not after it.
function finish(code, message) {
  if (message) (code === 0 ? console.log : console.error)(message);
  rmSync(LOCK_PATH, { force: true });
  process.exit(code);
}

async function main() {
  if (await health()) {
    console.log(`Portal already running (shared instance) on :${PORT}`);
    return;
  }

  mkdirSync(LOCK_DIR, { recursive: true });

  let haveLock = false;
  try {
    writeFileSync(LOCK_PATH, String(process.pid), { flag: 'wx' });
    haveLock = true;
  } catch {
    const stalePid = Number(readFileSync(LOCK_PATH, 'utf8').trim());
    if (!pidAlive(stalePid)) {
      rmSync(LOCK_PATH, { force: true });
      try {
        writeFileSync(LOCK_PATH, String(process.pid), { flag: 'wx' });
        haveLock = true;
      } catch {
        // Someone else grabbed it between our delete and retry — fall through to polling.
      }
    }
  }

  if (!haveLock) {
    console.log('Another process is already starting Portal — waiting for it...');
    const up = await pollHealth(180_000);
    if (up) {
      console.log(`Portal is up on :${PORT}`);
      return;
    }
    console.error('Timed out waiting for Portal to come up.');
    process.exitCode = 1;
    return;
  }

  console.log(`Building Portal (cargo build)... logging to ${LOG_PATH}`);
  const log = createWriteStream(LOG_PATH, { flags: 'a' });
  await new Promise((resolve, reject) => log.once('open', resolve).once('error', reject));
  log.write(`\n--- ${new Date().toISOString()} build ---\n`);

  const buildCode = await new Promise((resolve) => {
    const build = spawn('cargo', ['build'], { cwd: SRC_TAURI, stdio: ['ignore', 'pipe', 'pipe'] });
    build.stdout.pipe(log, { end: false });
    build.stderr.pipe(log, { end: false });
    build.on('exit', (code) => resolve(code ?? 1));
    build.on('error', (err) => {
      log.write(`cargo build failed to start: ${err.message}\n`);
      resolve(1);
    });
  });
  if (buildCode !== 0) {
    return finish(
      1,
      `cargo build failed (exit ${buildCode}).\n--- tail of ${LOG_PATH} ---\n${logTail()}`
    );
  }

  log.write(`\n--- ${new Date().toISOString()} run ${EXE_PATH} ---\n`);
  const app = spawn(EXE_PATH, [], {
    cwd: SRC_TAURI,
    stdio: ['ignore', 'pipe', 'pipe'],
    detached: true,
  });
  app.stdout.pipe(log);
  app.stderr.pipe(log);
  app.unref();

  const up = await pollHealth(180_000);
  if (up) return finish(0, `Portal is up on :${PORT}`);
  finish(
    1,
    `Portal did not come up within 180s after launch.\n--- tail of ${LOG_PATH} ---\n${logTail() || '(empty — the process may not have started at all)'}`
  );
}

main();
