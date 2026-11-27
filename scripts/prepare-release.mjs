#!/usr/bin/env node
/**
 * Release Portal Desktop. `.portal/pipeline.yml` runs this; the build gates live there.
 *
 *   node scripts/prepare-release.mjs <patch|minor|major|x.y.z>          dry run
 *   node scripts/prepare-release.mjs <patch|minor|major|x.y.z> --push   release
 *   node scripts/prepare-release.mjs --preflight                        could we release now?
 *
 * The bump may come from $BUMP instead. $RELEASE_DRY_RUN=1 wins over --push.
 *
 * A release syncs the version in package.json, tauri.conf.json, Cargo.toml and
 * Cargo.lock, adds a RELEASE_NOTES.md section from the commit titles since the last tag
 * (unless a section for that version is already committed), commits, tags vX.Y.Z and
 * pushes both to origin, where GitHub Actions builds and publishes it.
 */
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../', import.meta.url));
const BRANCH = 'main';

// Relative to ROOT. Also everything a release commit may contain; nothing else is staged.
const FILES = {
  packageJson: 'package.json',
  tauriConf: 'src-tauri/tauri.conf.json',
  cargoToml: 'src-tauri/Cargo.toml',
  cargoLock: 'src-tauri/Cargo.lock',
  releaseNotes: 'docs/status/RELEASE_NOTES.md',
};

const read = (rel) => readFileSync(join(ROOT, rel), 'utf8');
const write = (rel, text) => writeFileSync(join(ROOT, rel), text);

/** @param {string} cmd @param {string[]} args */
function run(cmd, args) {
  return execFileSync(cmd, args, {
    cwd: ROOT,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  }).trim();
}

/** @param {...string} args */
const git = (...args) => run('git', args);

/** @param {...string} args */
function gitSucceeds(...args) {
  try {
    git(...args);
    return true;
  } catch {
    return false;
  }
}

const tagExists = (tag) => gitSucceeds('rev-parse', '-q', '--verify', `refs/tags/${tag}`);

/** @param {string} version @param {string} bump patch, minor, major or an exact x.y.z */
function nextVersion(version, bump) {
  if (/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(bump)) return bump;

  const match = version.match(/^(\d+)\.(\d+)\.(\d+)/);
  if (!match) throw new Error(`Cannot bump version: ${version}`);
  const [major, minor, patch] = match.slice(1).map(Number);
  if (bump === 'major') return `${major + 1}.0.0`;
  if (bump === 'minor') return `${major}.${minor + 1}.0`;
  if (bump === 'patch') return `${major}.${minor}.${patch + 1}`;
  throw new Error(`Expected patch, minor, major or x.y.z, got "${bump}"`);
}

function currentVersion() {
  const versions = [
    JSON.parse(read(FILES.packageJson)).version,
    JSON.parse(read(FILES.tauriConf)).version,
    read(FILES.cargoToml).match(/^version\s*=\s*"([^"]+)"/m)?.[1],
  ];
  if (new Set(versions).size !== 1) throw new Error(`Version mismatch: ${versions.join(', ')}`);
  return versions[0];
}

/** @param {string} version */
function writeVersions(version) {
  // Patch the top-level (2-space indent) "version" line in place. Re-serialising with
  // JSON.stringify expands hand-formatted inline blocks and fails `prettier --check`.
  for (const file of [FILES.packageJson, FILES.tauriConf]) {
    const raw = read(file);
    const updated = raw.replace(/^( {2}"version"\s*:\s*)"[^"]*"/m, `$1"${version}"`);
    if (updated === raw) throw new Error(`Could not update version in ${file}`);
    write(file, updated);
  }

  const cargo = read(FILES.cargoToml);
  const updatedCargo = cargo.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
  if (updatedCargo === cargo) throw new Error(`Could not update version in ${FILES.cargoToml}`);
  write(FILES.cargoToml, updatedCargo);
}

/**
 * Refresh the `portal_desktop` entry in Cargo.lock to match Cargo.toml.
 *
 * The release workflow's smoke job runs `cargo check --locked`, which refuses
 * to update the lock. Bumping Cargo.toml without this leaves the lock one
 * version behind, the smoke job fails, and it gates all four platform builds —
 * so the tag publishes no binaries at all. This is what killed v0.7.0.
 *
 * @param {string} version
 */
function syncCargoLock(version) {
  run('cargo', ['update', '-p', 'portal_desktop', '--offline', '--manifest-path', FILES.cargoToml]);

  const entry = read(FILES.cargoLock).match(/name = "portal_desktop"\r?\nversion = "([^"]+)"/);
  if (entry?.[1] !== version) {
    throw new Error(
      `Cargo.lock still reports portal_desktop ${entry?.[1] ?? '?'} after update; expected ${version}. ` +
        `Release smoke (\`cargo check --locked\`) would fail.`
    );
  }
}

/** @param {string} version @param {string[]} notes */
function prependReleaseNotes(version, notes) {
  const current = read(FILES.releaseNotes);
  const marker = `## Version ${version}`;
  if (current.includes(marker)) return;

  const bullets = notes.map((line) => `- ${line}`).join('\n') || '- Maintenance release';
  const header = '# Portal Desktop - Release Notes\n\n';
  // \r?: a Windows autocrlf checkout has CRLF; a plain startsWith missed it and duplicated the header.
  const body = current.replace(/^# Portal Desktop - Release Notes\r?\n\r?\n/, '');

  write(FILES.releaseNotes, `${header}${marker}\n\n### Highlights\n\n${bullets}\n\n---\n\n${body}`);
}

/**
 * Why publishing would be wrong right now, or null when it is safe.
 *
 * @param {string} version the version currently in the files
 */
function publishBlocker(version) {
  const branch = git('branch', '--show-current');
  if (branch !== BRANCH) return `on branch "${branch}"; releases go from ${BRANCH}`;

  // .claude/ holds agent session notes, which are never part of a build.
  const dirty = git('status', '--porcelain', '--', '.', ':(exclude).claude');
  if (dirty) return `uncommitted changes:\n${dirty}`;

  git('fetch', 'origin', BRANCH);
  if (!gitSucceeds('merge-base', '--is-ancestor', `origin/${BRANCH}`, 'HEAD')) {
    return `behind origin/${BRANCH}; pull first`;
  }

  const last = `v${version}`;
  if (!tagExists(last)) {
    // Stopped between commit and tag: bumping again would skip a version.
    return git('log', '-1', '--pretty=%s') === `release: ${last}`
      ? `HEAD is the ${last} release commit but has no tag; run: git tag -a ${last} -m "Portal Desktop ${last}", then git push --atomic origin ${BRANCH} ${last}`
      : null;
  }
  if (!git('ls-remote', '--tags', 'origin', `refs/tags/${last}`)) {
    return `${last} is tagged here but not pushed; run: git push --atomic origin ${BRANCH} ${last}`;
  }
  return git('rev-list', '--count', `${last}..HEAD`) === '0' ? `nothing new since ${last}` : null;
}

/** @param {string} from @param {string} to */
function release(from, to) {
  const tag = `v${to}`;
  const notes = tagExists(`v${from}`)
    ? git('log', `v${from}..HEAD`, '--no-merges', '--pretty=%s').split('\n').filter(Boolean)
    : [];

  writeVersions(to);
  syncCargoLock(to);
  prependReleaseNotes(to, notes);

  git('add', '--', ...Object.values(FILES));
  git('commit', '-m', `release: ${tag}`);
  git('tag', '-a', tag, '-m', `Portal Desktop v${to}`);
  console.log(`Committed and tagged ${tag}`);

  try {
    // --atomic: a tag without its commit (or the reverse) never reaches origin.
    execFileSync('git', ['push', '--atomic', 'origin', BRANCH, `refs/tags/${tag}`], {
      cwd: ROOT,
      stdio: 'inherit',
    });
  } catch {
    throw new Error(
      `Push failed. ${tag} is committed and tagged locally; retry: git push --atomic origin ${BRANCH} ${tag}`
    );
  }
  console.log(`Pushed ${tag}. GitHub Actions builds and publishes it.`);
}

function main() {
  const args = process.argv.slice(2);
  const unknown = args.find((a) => a.startsWith('-') && a !== '--push' && a !== '--preflight');
  if (unknown) throw new Error(`Unknown flag: ${unknown}`);

  const from = currentVersion();

  if (args.includes('--preflight')) {
    const blocker = publishBlocker(from);
    if (blocker) throw new Error(`Cannot release: ${blocker}`);
    console.log('Preflight OK');
    return;
  }

  const bump = args.find((a) => !a.startsWith('-')) ?? process.env.BUMP;
  if (!bump) {
    throw new Error(
      'Usage: node scripts/prepare-release.mjs <patch|minor|major|x.y.z> [--push] | --preflight'
    );
  }
  const to = nextVersion(from, bump);
  if (to === from) throw new Error(`Already at ${from}`);
  console.log(`Release: ${from} -> ${to} (v${to})`);

  // Checked before any file is touched, so a refusal leaves a clean tree.
  const blocker =
    publishBlocker(from) ?? (tagExists(`v${to}`) ? `tag v${to} already exists` : null);

  if (!args.includes('--push') || process.env.RELEASE_DRY_RUN === '1') {
    if (blocker) console.log(`A real release would stop: ${blocker}`);
    console.log(
      `Dry run, nothing written. A release updates the version files and notes, commits, tags v${to} and pushes ${BRANCH}.`
    );
    return;
  }
  if (blocker) throw new Error(`Cannot release: ${blocker}`);
  release(from, to);
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
