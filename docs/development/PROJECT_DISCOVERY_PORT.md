# Project discovery — integration guide

Adds bulk project discovery, risk scoring, duplicate detection and triage to the
existing `projects` domain. The disk domain is untouched: its `DevCleaner` stays
the only thing that deletes, and discovery only _reports_ which regenerable
directories exist.

Wired in as of the commit that added it. The six integration edits listed below
are already applied; they are kept here as a map of the seams this feature
touches.

## Why this exists

`disk::scan_projects` already walks drives, but it builds its output from
cleanable items: a project with no `node_modules` produces no items and never
appears. It answers "what can I delete". This answers "what exists, is it alive,
and is any of its work only on this disk".

## New files

```
portal-core/src/domains/projects/discovery/mod.rs
portal-core/src/domains/projects/discovery/scan.rs          walk, dating, grouping
portal-core/src/domains/projects/discovery/git.rs           git probing
portal-core/src/domains/projects/discovery/analysis.rs      duplicates + risk
portal-core/src/domains/projects/repositories/discovery_repository.rs
portal-core/src/domains/projects/services/discovery_service.rs
portal-core/src/migrations/m20260817_000039_add_project_discovery_columns.rs
src/domains/projects/discovery_commands.rs                  Tauri adapter

src/lib/domains/projects/api/discoveryApi.ts
src/lib/domains/projects/components/ProjectAudit.svelte
src/routes/projects/audit/+page.svelte
```

## The six edits to existing files (applied)

**1. `portal-core/src/domains/projects/mod.rs`** — add the module:

```rust
pub mod discovery;
```

**2. `portal-core/src/domains/projects/services/mod.rs`**

```rust
pub mod discovery_service;
```

**3. `portal-core/src/domains/projects/repositories/mod.rs`**

```rust
pub mod discovery_repository;
```

**4. `portal-core/src/migrations/mod.rs`** — import and append to the list, after
`addProjectIdToCoderThreads`:

```rust
use m20260817_000039_add_project_discovery_columns::Migration as addProjectDiscoveryColumns;
// …
Box::new(addProjectDiscoveryColumns),
```

**5. `portal-core/src/entities/project.rs`** — the discovery columns on `Model`,
after `last_commit`. `ProjectResponse` in
`portal-core/src/domains/projects/entities/project.rs` carries the same fields, so
the Audit view can read `discovery_path` and `triage_decision` back from
`get_all_projects`.

**6. `src/lib.rs`** — register state and commands:

```rust
// with the other .manage(...) calls
.manage(domains::projects::discovery_commands::DiscoveryControl::default())

// in generate_handler![…], next to the other project commands
domains::projects::discovery_commands::discover_projects,
domains::projects::discovery_commands::discover_and_import_projects,
domains::projects::discovery_commands::import_discovered_projects,
domains::projects::discovery_commands::cancel_discovery,
domains::projects::discovery_commands::set_project_triage,
domains::projects::discovery_commands::set_project_triage_bulk,
```

and in `src/domains/projects/mod.rs`:

```rust
pub mod discovery_commands;
pub use discovery_commands::*;
```

## Design notes worth knowing before you review

**Identity is `discovery_path`, not the absolute path.** A project is
`<root name>/<category>/<name>`, so moving `D:\production` to `D:\projects\production`
keeps every triage decision attached. Upserts fall back to matching on the
absolute path (case- and separator-insensitive) so projects you added by hand get
_adopted_ rather than duplicated — the import result reports those separately.

**Activity ignores `.git` and build output.** This is not incidental. Probing a
repo with `git status` rewrites `.git/index`, and a naive newest-file check then
reports every repo as modified today. The same trap catches `npm install`. Dating
uses the newest source file or the last commit, whichever is later.

**Risk severity is about where work lives, not how messy it is.**
`high` = unpushed commits, or history with no remote — the work exists only on
that disk. `med` = uncommitted files, stashes, or active work outside version
control. A repo with no commits and no remote is deliberately _not_ high: there
is nothing to lose yet.

**`ahead` of `-1` means "no upstream configured"**, which is a different state
from `0` ("nothing to push"). The UI must not treat them alike.

**Grouping collapses at `project_depth`.** With the default of 2, a 45-module
course bundle is one row to triage rather than 45. Monorepo containers
(`apps/`, `packages/`, `crates/`, …) are the exception and stay separate, so
`pocket_portal/apps/portal_launcher` keeps its own unpushed-commit count.

**Missing projects are archived, never deleted.** A folder can be missing because
a drive is unplugged; destroying the row would take its triage note with it.
Enabling that at all requires passing `markMissing: true`.

## Tests

`analysis.rs` and `scan.rs` carry unit tests covering the pruning rules, monorepo
descent, depth grouping, the build-output dating trap, risk severity, and
duplicate-stem collapsing:

```bash
cargo test -p portal-core discovery
```

The engine was validated against `D:\practice\java` before hand-off and
reproduced an independent scan exactly: 53 roots, 9 projects, `spring-boot`
grouping 45 modules.

## Known limits

- Cancellation lands between project roots, not inside a single directory walk,
  so a cancel during one enormous tree waits for that tree.
- `git status` runs per repo and is the slowest call; set `skipGit: true` for a
  fast structural refresh.
- Sizes are not measured here on purpose — the disk domain already does it well,
  and doing it during discovery would multiply the walk cost.
