// What any Claude session/agent should run instead of blindly calling
// `pnpm tauri:dev` when it just wants Portal's shared daemon (port 18420) up
// and reachable — see this repo's CLAUDE.md. Idempotent and lock-protected:
// safe to call from multiple sessions/agents at once.
//
//   node scripts/ensure-portal.mjs
//
// Launches the INSTALLED RELEASE, never a build from this repo. The board is
// real data and lives in the release's database. A debug build runs under its
// own identifier, data dir and daemon port (18421) — see portal-core's
// app_paths.rs — so it can never answer here or migrate the release's
// database. `PORTAL_EXE` overrides where the release is looked for.
//
// ponytail: a lock file + PID liveness check is a simpler guard than a real
// mutex/IPC mechanism — good enough for "don't launch two of these on one
// machine". Upgrade to something more robust only if this races in practice.
import { spawn } from 'node:child_process';
import {
  createWriteStream,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { homedir, tmpdir } from 'node:os';
import { dirname, join } from 'node:path';

// Where the Windows installer puts a per-user install. Elsewhere: set PORTAL_EXE.
const EXE_PATH =
  process.env.PORTAL_EXE ??
  (process.platform === 'win32' && process.env.LOCALAPPDATA
    ? join(process.env.LOCALAPPDATA, 'Portal Desktop', 'portal_desktop.exe')
    : '');

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

  if (!EXE_PATH || !existsSync(EXE_PATH)) {
    console.error(
      `No installed Portal Desktop at ${EXE_PATH || '(unknown path)'}.\n` +
        'Install the release build, or set PORTAL_EXE to its executable.'
    );
    process.exitCode = 1;
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
    const up = await pollHealth(60_000);
    if (up) {
      console.log(`Portal is up on :${PORT}`);
      return;
    }
    console.error('Timed out waiting for Portal to come up.');
    process.exitCode = 1;
    return;
  }

  const log = createWriteStream(LOG_PATH, { flags: 'a' });
  await new Promise((resolve, reject) => log.once('open', resolve).once('error', reject));

  log.write(`\n--- ${new Date().toISOString()} run ${EXE_PATH} ---\n`);
  const app = spawn(EXE_PATH, [], {
    cwd: dirname(EXE_PATH),
    stdio: ['ignore', 'pipe', 'pipe'],
    detached: true,
  });
  app.stdout.pipe(log);
  app.stderr.pipe(log);
  app.unref();

  const up = await pollHealth(60_000);
  if (up) return finish(0, `Portal is up on :${PORT}`);
  finish(
    1,
    `Portal did not come up within 60s after launch.\n--- tail of ${LOG_PATH} ---\n${logTail() || '(empty — the process may not have started at all)'}`
  );
}

main();
