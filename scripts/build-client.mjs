// Builds the SvelteKit frontend in thin-client mode to its own output dir
// (`build-client/`, not `build/`) so it never collides with the desktop
// build — the daemon serves this to browsers and other-desktop clients.
// Own script, not an inline package.json env assignment, so it works on
// Windows and POSIX without cross-env (same reasoning as mobile/frontend.mjs).
import { spawn } from 'node:child_process';
import { writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const root = fileURLToPath(new URL('..', import.meta.url));
const outDir = join(root, 'build-client');
const child = spawn('pnpm', ['build'], {
  stdio: 'inherit',
  shell: true,
  env: { ...process.env, VITE_PORTAL_MOBILE: '1', VITE_BUILD_OUT_DIR: 'build-client' },
});
child.on('exit', (code) => {
  if (code === 0) {
    // adapter-static wipes the output dir before writing, taking `.gitkeep` with
    // it — put it back so a clean `cargo check` after this build still finds it.
    writeFileSync(join(outDir, '.gitkeep'), '');
  }
  process.exit(code ?? 1);
});
