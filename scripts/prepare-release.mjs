#!/usr/bin/env node
/**
 * Prepare a Portal Desktop release:
 * - sync semver across package.json, tauri.conf.json, and Cargo.toml
 * - optionally prepend a RELEASE_NOTES.md section
 * - optionally create an annotated git tag vX.Y.Z
 *
 * Usage:
 *   node scripts/prepare-release.mjs patch
 *   node scripts/prepare-release.mjs minor
 *   node scripts/prepare-release.mjs major
 *   node scripts/prepare-release.mjs 1.2.3
 *   node scripts/prepare-release.mjs current
 *   node scripts/prepare-release.mjs            (takes the bump from $BUMP)
 *
 * Flags:
 *   --dry-run     Print actions without writing files or creating tags (also $RELEASE_DRY_RUN=1)
 *   --preflight   Only check that a release could be published now
 *   --commit      Commit the version files and tag that commit
 *   --push        --commit, then push main and the tag (what .portal/pipeline.yml ships with)
 *   --no-tag      Skip git tag creation
 *   --no-notes    Skip RELEASE_NOTES.md stub
 *   --message/-m  Custom one-line note for RELEASE_NOTES (repeatable)
 */
import { execFileSync, execSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../', import.meta.url));

const FILES = {
  packageJson: join(ROOT, 'package.json'),
  tauriConf: join(ROOT, 'src-tauri/tauri.conf.json'),
  cargoToml: join(ROOT, 'src-tauri/Cargo.toml'),
  cargoLock: join(ROOT, 'src-tauri/Cargo.lock'),
  releaseNotes: join(ROOT, 'docs/status/RELEASE_NOTES.md'),
};

const BRANCH = 'main';

// Everything a release commit may contain; nothing else is ever staged.
const RELEASE_FILES = [
  'package.json',
  'src-tauri/tauri.conf.json',
  'src-tauri/Cargo.toml',
  'src-tauri/Cargo.lock',
  'docs/status/RELEASE_NOTES.md',
];

/** @param {...string} args */
function git(...args) {
  return execFileSync('git', args, {
    cwd: ROOT,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  }).trim();
}

/** @param {...string} args */
function gitSucceeds(...args) {
  try {
    git(...args);
    return true;
  } catch {
    return false;
  }
}

/** @param {string} cmd */
function run(cmd) {
  return execSync(cmd, { cwd: ROOT, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
}

/** @param {string} version */
function assertSemver(version) {
  if (!/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error(`Invalid semver: ${version}`);
  }
}

/** @param {string} version @param {"patch"|"minor"|"major"} part */
function bump(version, part) {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)/);
  if (!match) throw new Error(`Cannot bump version: ${version}`);

  let [major, minor, patch] = match.slice(1).map(Number);
  if (part === 'major') {
    major += 1;
    minor = 0;
    patch = 0;
  } else if (part === 'minor') {
    minor += 1;
    patch = 0;
  } else {
    patch += 1;
  }

  return `${major}.${minor}.${patch}`;
}

function readVersions() {
  const pkg = JSON.parse(readFileSync(FILES.packageJson, 'utf8'));
  const tauri = JSON.parse(readFileSync(FILES.tauriConf, 'utf8'));
  const cargo = readFileSync(FILES.cargoToml, 'utf8');
  const cargoMatch = cargo.match(/^version\s*=\s*"([^"]+)"/m);

  return {
    packageJson: pkg.version,
    tauriConf: tauri.version,
    cargoToml: cargoMatch?.[1] ?? null,
  };
}

/** @param {string} version */
function writeVersions(version) {
  // Patch the top-level (2-space indent) "version" line in place. Re-serialising with
  // JSON.stringify expands hand-formatted inline blocks and fails `prettier --check`.
  for (const file of [FILES.packageJson, FILES.tauriConf]) {
    const raw = readFileSync(file, 'utf8');
    const updated = raw.replace(/^(  "version"\s*:\s*)"[^"]*"/m, `$1"${version}"`);
    if (updated === raw) throw new Error(`Could not update version in ${file}`);
    writeFileSync(file, updated);
  }

  const cargo = readFileSync(FILES.cargoToml, 'utf8');
  const updatedCargo = cargo.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
  if (updatedCargo === cargo) {
    throw new Error('Could not update version in src-tauri/Cargo.toml');
  }
  writeFileSync(FILES.cargoToml, updatedCargo);
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
  run('cargo update -p portal_desktop --offline --manifest-path src-tauri/Cargo.toml');

  const lock = readFileSync(FILES.cargoLock, 'utf8');
  const entry = lock.match(/name = "portal_desktop"\nversion = "([^"]+)"/);
  if (entry?.[1] !== version) {
    throw new Error(
      `Cargo.lock still reports portal_desktop ${entry?.[1] ?? '?'} after update; expected ${version}. ` +
        `Release smoke (\`cargo check --locked\`) would fail.`
    );
  }

  console.log('Synced Cargo.lock');
}

/** @param {string} version @param {string[]} notes */
function prependReleaseNotes(version, notes) {
  const current = readFileSync(FILES.releaseNotes, 'utf8');
  const marker = `## Version ${version}`;

  if (current.includes(marker)) {
    console.log(`Release notes already contain ${marker}; skipping notes stub.`);
    return false;
  }

  const bullets = notes.length > 0 ? notes.map((line) => `- ${line}`).join('\n') : '- ';

  const section = `## Version ${version}

### Highlights

${bullets}

---

`;

  const header = '# Portal Desktop - Release Notes\n\n';
  // \r?: a Windows autocrlf checkout has CRLF; a plain startsWith missed it and duplicated the header.
  const body = current.replace(/^# Portal Desktop - Release Notes\r?\n\r?\n/, '');

  writeFileSync(FILES.releaseNotes, `${header}${section}${body}`);
  return true;
}

/** @param {string} tag */
function tagExists(tag) {
  try {
    run(`git rev-parse -q --verify "refs/tags/${tag}"`);
    return true;
  } catch {
    return false;
  }
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
  // Stopped between commit and tag: bumping again would skip a version.
  if (!tagExists(last) && git('log', '-1', '--pretty=%s') === `release: ${last}`) {
    return `HEAD is the ${last} release commit but has no tag; run: git tag -a ${last} -m "Portal Desktop ${last}", then git push --atomic origin ${BRANCH} ${last}`;
  }
  if (tagExists(last)) {
    if (!git('ls-remote', '--tags', 'origin', `refs/tags/${last}`)) {
      return `${last} is tagged here but not pushed; run: git push --atomic origin ${BRANCH} ${last}`;
    }
    if (git('rev-list', '--count', `${last}..HEAD`) === '0') return `nothing new since ${last}`;
  }
  return null;
}

/** @param {string} version @param {boolean} dryRun */
function preflight(version, dryRun) {
  const blocker = publishBlocker(version);
  if (blocker && !dryRun) throw new Error(`Cannot release: ${blocker}`);
  console.log(blocker ? `[dry-run] a real release would stop: ${blocker}` : 'Preflight OK');
}

/** @param {string} tag @param {boolean} dryRun */
function commitRelease(tag, dryRun) {
  if (dryRun) {
    console.log(`[dry-run] would commit "release: ${tag}"`);
    return;
  }
  git('add', '--', ...RELEASE_FILES);
  // `current` can leave nothing to commit: HEAD is then already the release.
  if (!gitSucceeds('diff', '--cached', '--quiet')) {
    git('commit', '-m', `release: ${tag}`);
    console.log(`Committed release: ${tag}`);
  }
}

/** @param {string} tag @param {boolean} dryRun */
function pushRelease(tag, dryRun) {
  if (dryRun) {
    console.log(`[dry-run] would push ${BRANCH} and ${tag} to origin`);
    return;
  }
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

/** @param {string} tag @param {string} version @param {boolean} dryRun */
function createTag(tag, version, dryRun) {
  if (tagExists(tag)) {
    throw new Error(`Git tag ${tag} already exists`);
  }

  if (dryRun) {
    console.log(`[dry-run] would create tag ${tag}`);
    return;
  }

  run(`git tag -a ${tag} -m "Portal Desktop v${version}"`);
  console.log(`Created tag ${tag}`);
}

function parseArgs(argv) {
  const flags = {
    dryRun: false,
    noTag: false,
    noNotes: false,
    preflight: false,
    commit: false,
    push: false,
    notes: [],
  };

  const positional = [];

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === '--dry-run') flags.dryRun = true;
    else if (arg === '--no-tag') flags.noTag = true;
    else if (arg === '--no-notes') flags.noNotes = true;
    else if (arg === '--preflight') flags.preflight = true;
    else if (arg === '--commit') flags.commit = true;
    else if (arg === '--push') flags.push = flags.commit = true;
    else if (arg === '--message' || arg === '-m') {
      const value = argv[++i];
      if (!value) throw new Error(`${arg} requires a value`);
      flags.notes.push(value);
    } else if (arg.startsWith('-')) {
      throw new Error(`Unknown flag: ${arg}`);
    } else {
      positional.push(arg);
    }
  }

  return { flags, positional };
}

function main() {
  const { flags, positional } = parseArgs(process.argv.slice(2));
  const bumpArg = positional[0] ?? process.env.BUMP;
  if (process.env.RELEASE_DRY_RUN === '1') flags.dryRun = true;

  if (flags.preflight) {
    preflight(readVersions().packageJson, flags.dryRun);
    return;
  }

  if (flags.commit && flags.noTag) {
    throw new Error('--commit and --push always tag; drop --no-tag');
  }

  if (!bumpArg) {
    console.error(`Usage: node scripts/prepare-release.mjs <patch|minor|major|x.y.z|current> [flags]

Examples:
  pnpm release:prepare patch
  pnpm release:prepare 1.0.0 -m "New feature"
  pnpm release:prepare current --no-notes

After running without --push:
  git add -A
  git commit -m "release: vX.Y.Z"
  git tag -a vX.Y.Z -m "Portal Desktop vX.Y.Z"
  git push --atomic origin main vX.Y.Z
`);
    process.exit(1);
  }

  const current = readVersions();
  const unique = new Set(Object.values(current).filter(Boolean));
  if (unique.size > 1) {
    throw new Error(`Version mismatch before release prep: ${JSON.stringify(current)}`);
  }

  const fromVersion = current.packageJson;
  let nextVersion = fromVersion;

  if (bumpArg === 'current') {
    console.log(`Using current version ${fromVersion}`);
  } else if (['patch', 'minor', 'major'].includes(bumpArg)) {
    nextVersion = bump(fromVersion, /** @type {"patch"|"minor"|"major"} */ (bumpArg));
  } else {
    assertSemver(bumpArg);
    nextVersion = bumpArg;
  }

  assertSemver(nextVersion);
  const tag = `v${nextVersion}`;

  console.log(`Release target: ${fromVersion} -> ${nextVersion} (${tag})`);

  // Both checks run before any file is touched, so a refusal leaves a clean tree.
  if (flags.commit) preflight(fromVersion, flags.dryRun);
  if (!flags.noTag && tagExists(tag)) throw new Error(`Git tag ${tag} already exists`);

  if (flags.commit && flags.notes.length === 0 && tagExists(`v${fromVersion}`)) {
    const log = git('log', `v${fromVersion}..HEAD`, '--no-merges', '--pretty=%s');
    flags.notes = log.split('\n').filter(Boolean);
  }

  if (nextVersion !== fromVersion) {
    if (flags.dryRun) {
      console.log('[dry-run] would update package.json, tauri.conf.json, Cargo.toml, Cargo.lock');
    } else {
      writeVersions(nextVersion);
      console.log('Updated version files');
      syncCargoLock(nextVersion);
    }
  } else {
    console.log('Version files already at target version');
  }

  let wroteNotes = false;
  if (!flags.noNotes) {
    if (flags.dryRun) {
      console.log('[dry-run] would prepend RELEASE_NOTES.md stub if missing');
      wroteNotes = true;
    } else {
      wroteNotes = prependReleaseNotes(nextVersion, flags.notes);
    }
  }

  if (flags.commit) commitRelease(tag, flags.dryRun);

  // A tag marks a commit, so with uncommitted edits it would land on the previous one.
  const changed = nextVersion !== fromVersion || wroteNotes;
  const tagged = !flags.noTag && (flags.commit || !changed);
  if (tagged) createTag(tag, nextVersion, flags.dryRun);

  if (flags.push) {
    pushRelease(tag, flags.dryRun);
    return;
  }

  const steps = [];
  if (!flags.commit) {
    if (changed) {
      steps.push('Review docs/status/RELEASE_NOTES.md', 'git add -A');
      steps.push(`git commit -m "release: ${tag}"`);
    }
    steps.push(
      'pnpm release:verify  # runs the release-smoke job locally via act — fix before tagging'
    );
    if (!tagged || flags.dryRun)
      steps.push(`git tag -a ${tag} -m "Portal Desktop v${nextVersion}"`);
  }
  steps.push(`git push --atomic origin ${BRANCH} ${tag}`);

  console.log('');
  console.log('Next steps:');
  steps.forEach((step, i) => console.log(`  ${i + 1}. ${step}`));
  console.log('');
  console.log('GitHub Actions will build and publish the release when the tag is pushed.');
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
