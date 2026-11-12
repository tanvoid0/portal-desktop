// Runs the root SvelteKit frontend (`pnpm dev` | `pnpm build`) in mobile mode.
// Own script so it works on Windows and POSIX without cross-env.
import { spawn } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('..', import.meta.url));
const child = spawn('pnpm', [process.argv[2] ?? 'build'], {
  cwd: root,
  stdio: 'inherit',
  shell: true,
  env: { ...process.env, VITE_PORTAL_MOBILE: '1' },
});
child.on('exit', (code) => process.exit(code ?? 1));
