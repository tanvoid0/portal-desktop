// Wraps `tauri android <args>` from mobile/, then re-patches BuildTask.kt.
//
// `tauri android init` (and re-init on build) bakes the wrong self-invocation
// into gen/android/buildSrc/.../BuildTask.kt when run through a package-manager
// shim (pnpm/npm/yarn): tauri.js's binName detection (see its npm_execpath
// branch) produces something unusable for Gradle's execOperations, so the
// generated Kotlin ends up calling a bare `tauri` that Gradle can't resolve.
// gen/android is gitignored (generated, machine-specific), so the fix can't be
// committed there — it has to be re-applied after every regen. This script is
// the "after every regen" part: real fix, not a workaround.
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import path from 'node:path';

const mobileDir = fileURLToPath(new URL('.', import.meta.url));
const repoRoot = fileURLToPath(new URL('..', import.meta.url));
const tauriJs = path.join(repoRoot, 'node_modules/@tauri-apps/cli/tauri.js');

const result = spawnSync(process.execPath, [tauriJs, 'android', ...process.argv.slice(2)], {
  cwd: mobileDir,
  stdio: 'inherit',
});

patchBuildTask();

process.exit(result.status ?? 1);

function patchBuildTask() {
  const file = path.join(
    mobileDir,
    'src-tauri/gen/android/buildSrc/src/main/java/com/tanvoid0/portal/mobile/kotlin/BuildTask.kt'
  );
  if (!existsSync(file)) return;

  const tauriJsPosix = tauriJs.replace(/\\/g, '/');
  const src = readFileSync(file, 'utf8');
  const patched = src.replace(
    /val args = listOf\("[^"]*",\s*"android",\s*"android-studio-script"\)/,
    `val args = listOf("${tauriJsPosix}", "android", "android-studio-script")`
  );
  if (patched !== src) writeFileSync(file, patched);
}
