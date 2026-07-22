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
 *
 * Flags:
 *   --dry-run     Print actions without writing files or creating tags
 *   --no-tag      Skip git tag creation
 *   --no-notes    Skip RELEASE_NOTES.md stub
 *   --message/-m  Custom one-line note for RELEASE_NOTES (repeatable)
 */
import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = fileURLToPath(new URL("../", import.meta.url));

const FILES = {
  packageJson: join(ROOT, "package.json"),
  tauriConf: join(ROOT, "src-tauri/tauri.conf.json"),
  cargoToml: join(ROOT, "src-tauri/Cargo.toml"),
  cargoLock: join(ROOT, "src-tauri/Cargo.lock"),
  releaseNotes: join(ROOT, "docs/status/RELEASE_NOTES.md"),
};

/** @param {string} cmd */
function run(cmd) {
  return execSync(cmd, { cwd: ROOT, encoding: "utf8", stdio: ["ignore", "pipe", "pipe"] }).trim();
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
  if (part === "major") {
    major += 1;
    minor = 0;
    patch = 0;
  } else if (part === "minor") {
    minor += 1;
    patch = 0;
  } else {
    patch += 1;
  }

  return `${major}.${minor}.${patch}`;
}

function readVersions() {
  const pkg = JSON.parse(readFileSync(FILES.packageJson, "utf8"));
  const tauri = JSON.parse(readFileSync(FILES.tauriConf, "utf8"));
  const cargo = readFileSync(FILES.cargoToml, "utf8");
  const cargoMatch = cargo.match(/^version\s*=\s*"([^"]+)"/m);

  return {
    packageJson: pkg.version,
    tauriConf: tauri.version,
    cargoToml: cargoMatch?.[1] ?? null,
  };
}

/** @param {string} version */
function writeVersions(version) {
  const pkg = JSON.parse(readFileSync(FILES.packageJson, "utf8"));
  pkg.version = version;
  writeFileSync(FILES.packageJson, `${JSON.stringify(pkg, null, 2)}\n`);

  const tauri = JSON.parse(readFileSync(FILES.tauriConf, "utf8"));
  tauri.version = version;
  writeFileSync(FILES.tauriConf, `${JSON.stringify(tauri, null, 2)}\n`);

  const cargo = readFileSync(FILES.cargoToml, "utf8");
  const updatedCargo = cargo.replace(
    /^version\s*=\s*"[^"]+"/m,
    `version = "${version}"`,
  );
  if (updatedCargo === cargo) {
    throw new Error("Could not update version in src-tauri/Cargo.toml");
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
  run("cargo update -p portal_desktop --offline --manifest-path src-tauri/Cargo.toml");

  const lock = readFileSync(FILES.cargoLock, "utf8");
  const entry = lock.match(/name = "portal_desktop"\nversion = "([^"]+)"/);
  if (entry?.[1] !== version) {
    throw new Error(
      `Cargo.lock still reports portal_desktop ${entry?.[1] ?? "?"} after update; expected ${version}. ` +
        `Release smoke (\`cargo check --locked\`) would fail.`,
    );
  }

  console.log("Synced Cargo.lock");
}

/** @param {string} version @param {string[]} notes */
function prependReleaseNotes(version, notes) {
  const current = readFileSync(FILES.releaseNotes, "utf8");
  const marker = `## Version ${version}`;

  if (current.includes(marker)) {
    console.log(`Release notes already contain ${marker}; skipping notes stub.`);
    return false;
  }

  const bullets =
    notes.length > 0
      ? notes.map((line) => `- ${line}`).join("\n")
      : "- ";

  const section = `## Version ${version}

### Highlights

${bullets}

---

`;

  const header = "# Portal Desktop - Release Notes\n\n";
  const body = current.startsWith(header)
    ? current.slice(header.length)
    : current;

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
    notes: [],
  };

  const positional = [];

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--dry-run") flags.dryRun = true;
    else if (arg === "--no-tag") flags.noTag = true;
    else if (arg === "--no-notes") flags.noNotes = true;
    else if (arg === "--message" || arg === "-m") {
      const value = argv[++i];
      if (!value) throw new Error(`${arg} requires a value`);
      flags.notes.push(value);
    } else if (arg.startsWith("-")) {
      throw new Error(`Unknown flag: ${arg}`);
    } else {
      positional.push(arg);
    }
  }

  return { flags, positional };
}

function main() {
  const { flags, positional } = parseArgs(process.argv.slice(2));
  const bumpArg = positional[0];

  if (!bumpArg) {
    console.error(`Usage: node scripts/prepare-release.mjs <patch|minor|major|x.y.z|current> [flags]

Examples:
  pnpm release:prepare patch
  pnpm release:prepare 1.0.0 -m "New feature"
  pnpm release:prepare current --no-notes

After running:
  git add -A
  git commit -m "release: vX.Y.Z"
  git push origin main --tags
`);
    process.exit(1);
  }

  const current = readVersions();
  const unique = new Set(Object.values(current).filter(Boolean));
  if (unique.size > 1) {
    throw new Error(
      `Version mismatch before release prep: ${JSON.stringify(current)}`,
    );
  }

  const fromVersion = current.packageJson;
  let nextVersion = fromVersion;

  if (bumpArg === "current") {
    console.log(`Using current version ${fromVersion}`);
  } else if (["patch", "minor", "major"].includes(bumpArg)) {
    nextVersion = bump(fromVersion, /** @type {"patch"|"minor"|"major"} */ (bumpArg));
  } else {
    assertSemver(bumpArg);
    nextVersion = bumpArg;
  }

  assertSemver(nextVersion);
  const tag = `v${nextVersion}`;

  console.log(`Release target: ${fromVersion} -> ${nextVersion} (${tag})`);

  if (nextVersion !== fromVersion) {
    if (flags.dryRun) {
      console.log("[dry-run] would update package.json, tauri.conf.json, Cargo.toml, Cargo.lock");
    } else {
      writeVersions(nextVersion);
      console.log("Updated version files");
      syncCargoLock(nextVersion);
    }
  } else {
    console.log("Version files already at target version");
  }

  if (!flags.noNotes) {
    if (flags.dryRun) {
      console.log("[dry-run] would prepend RELEASE_NOTES.md stub if missing");
    } else {
      prependReleaseNotes(nextVersion, flags.notes);
    }
  }

  if (!flags.noTag) {
    createTag(tag, nextVersion, flags.dryRun);
  }

  console.log("");
  console.log("Next steps:");
  if (nextVersion !== fromVersion || !flags.noNotes) {
    console.log("  1. Review docs/status/RELEASE_NOTES.md");
    console.log("  2. git add -A");
    console.log(`  3. git commit -m "release: v${nextVersion}"`);
  } else {
    console.log("  1. git add -A");
    console.log(`  2. git commit -m "release: v${nextVersion}"`);
  }
  if (!flags.noTag && !flags.dryRun) {
    console.log(`  ${nextVersion !== fromVersion || !flags.noNotes ? 4 : 3}. git push origin main --tags`);
  } else {
    console.log(`  ${nextVersion !== fromVersion || !flags.noNotes ? 4 : 3}. git tag -a ${tag} -m "Portal Desktop v${nextVersion}"`);
    console.log(`  ${nextVersion !== fromVersion || !flags.noNotes ? 5 : 4}. git push origin main --tags`);
  }
  console.log("");
  console.log("GitHub Actions will build and publish the release when the tag is pushed.");
}

try {
  main();
} catch (error) {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
}
