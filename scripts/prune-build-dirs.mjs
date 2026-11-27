#!/usr/bin/env node
// Frees the per-checkout Rust build dirs that src-tauri/.cargo/config.toml's
// `build-dir = .../build/{workspace-path-hash}` leaves behind. Every checkout
// path (main + each worktree) x every crate dir (src-tauri, portal-core,
// rpc-macro - none declares [workspace], so each is its own) gets a 3-30 GB
// build dir that nothing deletes when the worktree goes. 2026-11-27: 53 orphans,
// ~420 GB, I: at 0 bytes free.
//
// Keeps the dirs `cargo metadata` names for every live checkout; deletes the
// rest, so run it right after `git worktree remove`. In the live ones, drops
// incremental caches untouched for 3 days.
//
//   pnpm prune:build            delete
//   pnpm prune:build --dry-run  list only
import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, rmSync, statSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';

const dry = process.argv.includes('--dry-run');
const HOUR = 3600e3;
const now = Date.now();
const key = (p) => resolve(p).toLowerCase();

const checkouts = execFileSync('git', ['worktree', 'list', '--porcelain'], { encoding: 'utf8' })
  .split('\n')
  .filter((l) => l.startsWith('worktree '))
  .map((l) => l.slice(9).trim());

const live = new Set();
let root;
for (const co of checkouts) {
  const tauri = join(co, 'src-tauri');
  if (!existsSync(tauri)) continue;
  const crates = [tauri, ...readdirSync(tauri).map((d) => join(tauri, d))].filter((d) =>
    existsSync(join(d, 'Cargo.toml'))
  );
  for (const cwd of crates) {
    // cwd matters: .cargo/config.toml is found from the working dir, not the manifest.
    const meta = JSON.parse(
      execFileSync('cargo', ['metadata', '--no-deps', '--format-version', '1'], {
        cwd,
        encoding: 'utf8',
      })
    );
    if (!meta.build_directory || key(meta.build_directory) === key(meta.target_directory)) continue;
    live.add(key(meta.build_directory));
    root ??= dirname(dirname(resolve(meta.build_directory)));
  }
}
if (!root) throw new Error('no checkout uses a separate build-dir; nothing to prune');

const mtime = (p) => (existsSync(p) ? statSync(p).mtimeMs : 0);
const gone = (p, why) => {
  console.log(`${dry ? 'would delete' : 'delete'}  ${p}  (${why})`);
  if (!dry) rmSync(p, { recursive: true, force: true, maxRetries: 3 });
};

for (const a of readdirSync(root)) {
  for (const b of readdirSync(join(root, a))) {
    const dir = join(root, a, b);
    if (live.has(key(dir))) {
      for (const profile of ['debug', 'release']) {
        const inc = join(dir, profile, 'incremental');
        if (!existsSync(inc)) continue;
        for (const c of readdirSync(inc)) {
          const p = join(inc, c);
          if (now - mtime(p) > 72 * HOUR) gone(p, 'incremental, idle 3d');
        }
      }
      continue;
    }
    // `git worktree add` registers before cargo can create the dir, so only a dir
    // born after the listing above can belong to a checkout this run did not see.
    if (statSync(dir).birthtimeMs >= now) console.log(`skip   ${dir}  (created during this run)`);
    else gone(dir, 'no live checkout');
  }
}
