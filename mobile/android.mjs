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
patchManifest();
patchMainActivity();

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

// Android 16 (One UI) gates LAN/private-IP access behind ACCESS_LOCAL_NETWORK;
// without it the OS silently drops the app's packets to the desktop daemon
// (192.168.x.x:18420) and pairing never reaches the PC. Tauri's generated
// manifest only declares INTERNET, and gen/android is gitignored/regenerated,
// so like BuildTask.kt the permission has to be re-added after every regen.
function patchManifest() {
  const file = path.join(mobileDir, 'src-tauri/gen/android/app/src/main/AndroidManifest.xml');
  if (!existsSync(file)) return;

  const perm = '<uses-permission android:name="android.permission.ACCESS_LOCAL_NETWORK" />';
  const src = readFileSync(file, 'utf8');
  if (src.includes('ACCESS_LOCAL_NETWORK')) return;

  const patched = src.replace(
    /(<uses-permission android:name="android\.permission\.INTERNET" \/>)/,
    `$1\n    ${perm}`
  );
  if (patched !== src) writeFileSync(file, patched);
}

// Declaring ACCESS_LOCAL_NETWORK is not enough: on Android 16 it is a runtime
// (dangerous) permission, so a fresh install has it denied and the LAN packets
// are still dropped until the user grants it. Fire the system request from
// MainActivity.onCreate (before any pairing/LAN call). MainActivity.kt is
// regenerated on init like the manifest, so re-apply after every regen.
function patchMainActivity() {
  const file = path.join(
    mobileDir,
    'src-tauri/gen/android/app/src/main/java/com/tanvoid0/portal/mobile/MainActivity.kt'
  );
  if (!existsSync(file)) return;

  const src = readFileSync(file, 'utf8');
  if (src.includes('ACCESS_LOCAL_NETWORK')) return;

  let patched = src.replace(
    /^import android\.os\.Bundle$/m,
    'import android.content.pm.PackageManager\nimport android.os.Build\nimport android.os.Bundle'
  );
  patched = patched.replace(
    /(super\.onCreate\(savedInstanceState\))/,
    `$1
    // Android 16+ gates LAN access behind ACCESS_LOCAL_NETWORK (runtime perm).
    val lnp = "android.permission.ACCESS_LOCAL_NETWORK"
    if (Build.VERSION.SDK_INT >= 36 && checkSelfPermission(lnp) != PackageManager.PERMISSION_GRANTED) {
      requestPermissions(arrayOf(lnp), 0)
    }`
  );
  if (patched !== src) writeFileSync(file, patched);
}
