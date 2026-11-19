#!/usr/bin/env node
// Downloads the pinned agent-platformd release for a target triple into
// src-tauri/binaries/agent-platformd-<triple>[.exe] (Tauri `externalBin` naming).
// Run by `pnpm tauri:build` and by release CI; dev does not need it (see
// platform_daemon::daemon_exe).
//
// Target triple: TAURI_TARGET_TRIPLE if set (release CI sets it per matrix job to
// match tauri's `--target`), else the rustc host triple. The output file is named
// with THIS triple, because that is what Tauri looks for when bundling.
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import {
  chmodSync,
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

// agent-platform release tag whose agent-platformd this portal build bundles.
// Confirmed 2026-11-19: latest stable tag, cargo-dist 0.32.0 assets named
// `agent-platform-server-<triple>.tar.xz|.zip` (+ a `.sha256` sibling), binary
// `agent-platformd` inside. Bump in lockstep with a deliberate daemon upgrade.
const TAG = 'v0.7.6';
const REPO = 'tanvoid0/agent-platform';

// The triple Tauri will look for (= output filename) — may differ from the triple
// whose release asset we download (see DAEMON_TARGET below).
const outTriple =
  process.env.TAURI_TARGET_TRIPLE ??
  execFileSync('rustc', ['-vV'], { encoding: 'utf8' }).match(/host: (\S+)/)?.[1];
if (!outTriple) throw new Error('could not determine the target triple');

// agent-platform's dist-workspace.toml ships no aarch64-apple-darwin daemon
// (GitHub stopped granting that repo its macOS ARM runner); upstream's own stance
// is that Apple Silicon runs the x86_64 daemon under Rosetta 2. So for an arm64
// mac portal build, bundle the x86_64 daemon under the arm64 filename. Ceiling:
// the user's Mac needs Rosetta 2 (present by default on macOS 11+ once any x86
// app has run). Revisit if agent-platform restores an arm64 daemon asset.
const DAEMON_TARGET = { 'aarch64-apple-darwin': 'x86_64-apple-darwin' };
const assetTriple = DAEMON_TARGET[outTriple] ?? outTriple;

const exe = outTriple.includes('windows') ? '.exe' : '';
const out = join('src-tauri', 'binaries', `agent-platformd-${outTriple}${exe}`);
if (existsSync(out) && !process.env.FORCE) {
  console.log(`${out} already present (FORCE=1 to refetch)`);
  process.exit(0);
}

const headers = {
  'User-Agent': 'portal-desktop-build',
  ...(process.env.GITHUB_TOKEN && { Authorization: `Bearer ${process.env.GITHUB_TOKEN}` }),
};

const relRes = await fetch(`https://api.github.com/repos/${REPO}/releases/tags/${TAG}`, {
  headers,
});
if (!relRes.ok)
  throw new Error(`could not fetch ${REPO}@${TAG}: ${relRes.status} ${relRes.statusText}`);
const rel = await relRes.json();

// cargo-dist names the daemon archive after the PACKAGE (`agent-platform-server`),
// not the binary; the `agent-platform-desktop-*.zip` (iced app) shares the Windows
// triple, so anchor on the exact prefix or that app is picked up instead.
const assetName = (ext) => `agent-platform-server-${assetTriple}.${ext}`;
const archiveExts = ['tar.xz', 'zip', 'tar.gz'];
const asset = rel.assets?.find((a) => archiveExts.some((ext) => a.name === assetName(ext)));
if (!asset) {
  const want = archiveExts.map((e) => assetName(e)).join(' | ');
  throw new Error(
    `no agent-platformd asset for ${assetTriple} in ${REPO}@${TAG}\n` +
      `  expected one of: ${want}\n` +
      `  available: ${(rel.assets ?? []).map((a) => a.name).join(', ') || '(none)'}`
  );
}

const download = async (url) => {
  const res = await fetch(url, { headers });
  if (!res.ok) throw new Error(`download failed for ${url}: ${res.status} ${res.statusText}`);
  return Buffer.from(await res.arrayBuffer());
};

const work = mkdtempSync(join(tmpdir(), 'agentplatformd-'));
try {
  const bytes = await download(asset.browser_download_url);

  // cargo-dist publishes `<asset>.sha256` (format: `<hex> *<name>`). Verify before
  // trusting the archive, so a corrupt or tampered download fails the build loudly
  // rather than shipping.
  const sumAsset = rel.assets.find((a) => a.name === `${asset.name}.sha256`);
  if (sumAsset) {
    const expected = (await download(sumAsset.browser_download_url))
      .toString('utf8')
      .trim()
      .split(/\s+/)[0]
      .toLowerCase();
    const actual = createHash('sha256').update(bytes).digest('hex');
    if (actual !== expected)
      throw new Error(
        `checksum mismatch for ${asset.name}\n  expected ${expected}\n  got      ${actual}`
      );
    console.log(`checksum ok (${asset.name})`);
  } else {
    console.warn(`WARN: no ${asset.name}.sha256 published; skipping checksum verification`);
  }

  const archive = join(work, asset.name);
  writeFileSync(archive, bytes);
  // Extract. On Windows the asset is a .zip, which git-bash's GNU `tar` (first on a
  // bash shell's PATH in CI) cannot read; Windows ships bsdtar at System32\tar.exe
  // which handles both .zip and .tar.xz, so call it explicitly. Elsewhere plain
  // `tar` unpacks .tar.xz. Run in `work` with a bare basename: GNU tar reads a
  // drive-letter path (`C:\...`) as a remote `host:path`.
  const tarBin =
    process.platform === 'win32'
      ? join(process.env.SystemRoot ?? 'C:\\Windows', 'System32', 'tar.exe')
      : 'tar';
  execFileSync(tarBin, ['-xf', asset.name], { cwd: work });
  const find = (dir) =>
    readdirSync(dir).flatMap((n) => {
      const p = join(dir, n);
      return statSync(p).isDirectory() ? find(p) : [p];
    });
  // The output always carries the Windows `.exe` suffix when building for Windows,
  // regardless of which triple's asset we unpacked.
  const assetExe = assetTriple.includes('windows') ? '.exe' : '';
  const bin = find(work).find((p) => p.endsWith(`agent-platformd${assetExe}`));
  if (!bin) throw new Error(`agent-platformd${assetExe} not found inside ${asset.name}`);
  mkdirSync(join('src-tauri', 'binaries'), { recursive: true });
  copyFileSync(bin, out);
  if (!exe) chmodSync(out, 0o755);
  console.log(`fetched ${asset.name} -> ${out}`);
} finally {
  rmSync(work, { recursive: true, force: true });
}
