# Portal Desktop trial run — building an Expo todo app end-to-end

**Date:** 2026-11-11
**Goal:** Act as a developer and build a React Native **Expo** todo-list app using Portal
Desktop's own **built-in coder**, **Projects**, and **CI/CD (Actions/pipelines)** — driving
the real app over CDP (Playwright on `:9223`, instance-2), with as little manual
intervention as possible, to measure how much of the build→test→release loop the app
actually automates and where the gaps are.

**One-line verdict:** The **coder is the standout** — it built the whole app hands-off with
zero approval prompts. The surrounding **project → CI/CD → release chain is wired but not
trustworthy yet**: commands are not Expo-aware, the package-manager choice broke every
pipeline on this machine, in-app action runs gave no feedback, and the app exited mid-trial
and could not be restarted from source.

---

## What actually got built

A working Expo + TypeScript todo app at `D:\projects\trials\expo-todo` (Portal project
**"Expo Todo"**, id 201):

- `App.tsx` — text input + Add, `FlatList`, tap-to-toggle (completed = strikethrough), ✕
  delete, persisted via `@react-native-async-storage/async-storage` (survives restart).
- `todos.ts` — pure add/toggle/delete/list reducer (no React imports).
- `todos.test.ts` — one jest-expo test (add / blank-skip / toggle / delete).
- `test` script wired into `package.json`.

**Coder self-verified in-session:** `npx tsc --noEmit` clean, `npm test` → **1 passed.**
(The coder left its changes uncommitted — correct; the only commit is create-expo's own
"Initial commit" under the user's git identity, no stray AI attribution.)

---

## What worked well (the automation wins)

1. **Project creation is fully automatable.** New Project's path field (`FolderPicker`)
   accepts a typed path — no native dialog required — and `analyze_project_directory`
   auto-detects framework/language/PM/commands. Created the project over CDP with one
   `fill()` + submit; it auto-grouped under `trials/` from the parent folder.

2. **The built-in coder (Claude-CLI engine, "Auto" mode) is genuinely hands-off.** From a
   single-line prompt it ran the entire loop with **zero approval prompts**:
   `create-expo-app` (auto-detected pnpm) → installed async-storage + jest-expo → wrote the
   screen, the pure reducer, and a test → ran `tsc` and `npm test` and confirmed both pass.
   This is the strongest part of the product.

3. **Coder ↔ project binding is clean.** A coder session's workspace is chosen from the
   Projects list, so project + coder + pipelines all converge on one directory. Seeding
   `localStorage['portal-coder-last-workspace']` pre-selected the right project.

4. **CI/CD intent is auto-scaffolded.** From project metadata the Actions tab generated
   Install / Dev / Build / Test / Lint actions **and** a ready **CI workflow**
   (Install → Test → Build), plus a **"Create release file"** generator. The `dev` action
   correctly resolved to `pnpm run start` (i.e. `expo start`) from the detected start
   command — nice.

---

## Findings / issues (most severe first)

### 1. [High] The app exited on its own mid-trial and cannot be restarted from source

The running instance-2 (a pre-built debug binary) **exited by itself** shortly after the
coder session finished — no user action. Attempting a clean restart with `tauri dev`
rebuilds from the working tree and **fails to compile**:
`error[E0277]: the trait bound 'Caller: serde::de::DeserializeOwned' is not satisfied`
(repeated). So once the running binary is gone there is no quick way back into the app.
_Impact:_ blocked the live CI/CD portion entirely. _Enhancement:_ keep `main`'s working
tree buildable; surface a crash/exit reason; consider a "relaunch last good build" that
doesn't force a full recompile.

### 2. [High] Auto-generated CI/CD commands are not Expo/React-Native aware

The project was detected as **"React"** (not Expo/React Native). The default actions use
generic package.json script names:

- `Build → pnpm build` — **no `build` script exists** in an Expo app (confirmed: "Missing
  script"). The CI workflow fails at its Build step.
- `Lint → pnpm lint` — **no `lint` script** either.
- Expo's real equivalents are `expo export` / `eas build` and `expo lint`.

_Impact:_ the out-of-the-box CI workflow is red on a brand-new Expo app. _Enhancement:_
detect Expo/React-Native and map build→`expo export`/`eas build`, lint→`expo lint`, plus a
`tsc --noEmit` typecheck action (there is none today).

### 3. [High] Auto-picked package manager (pnpm) made every pipeline unrunnable here

Detection chose **pnpm**, so all actions are `pnpm …`. On this machine pnpm is configured
with a lockfile policy that **rejects the create-expo lockfile**:

> "The lockfile contains entries that the active policies reject … run `pnpm clean
--lockfile` and then `pnpm install`."
> Because pnpm auto-runs a deps-status check (`pnpm install`) before any script, **test,
> build and lint all fail at that gate before the real command runs.** Non-TTY runners also
> need `CI=true` or pnpm prompts for a modules purge. The coder only succeeded because it used
> `npm test` / `npx tsc`, bypassing pnpm.
> _Impact:_ Portal's whole Actions surface is dead-on-arrival for this project on this
> machine. _Enhancement:_ let the user pick/override the PM per project; fall back to npm when
> pnpm's preflight fails; run actions with `CI=true` in the non-interactive runner.

### 4. [Med] No "new app" scaffolder — project flow can't bootstrap, only register

New Project only _registers an existing directory_; there is no "create app from framework"
(e.g. `create-expo-app`) path. A brand-new app must be bootstrapped elsewhere first. The
coder filled this gap, but there's no first-class **project → scaffold → open-in-coder**
chain. _Enhancement:_ a "scaffold with…" option that creates the dir, runs the starter, and
opens the coder on it.

### 5. [Med] Project metadata is captured once and goes stale

Detection ran at create-time against the **empty** folder, so after the coder scaffolded a
real git repo the project still shows **"This folder is not a git repository"** and only
"React/TypeScript/pnpm". There's no automatic re-sync when the folder changes (Sync is
manual and buried in the edit form). _Enhancement:_ re-analyze on focus / after a coder
session, or show a "metadata is stale — Sync" nudge.

### 6. [Med] In-app Action "Run" gave no visible feedback

Clicking an action's **Run** never populated "Local run history" (stayed "No action runs
yet") and showed no inline output/spinner/error — success and failure look identical.
(Likely the run died at pnpm's policy gate, but the UI said nothing.) _Enhancement:_ stream
output inline, show running/exit state, and surface non-zero exits loudly.

### 7. [Low] Misleading loading states

`/projects` first-paint showed **"Total 0 / Loading projects…"** on a DB with **196**
projects; `/projects/201` hung on **"Loading project details…"** while `/projects/201/pipelines`
loaded fine. Empty/loading states read as "nothing here." _Enhancement:_ skeletons with real
counts; fix the detail route's hang.

### 8. [Low] Stale cross-platform cached workspace

The coder's cached default workspace was a **Linux path `/media/tan/Workspace/...`** on a
Windows box (persisted in localStorage). A fresh session pre-targets a non-existent
directory. _Enhancement:_ validate the cached path exists before defaulting to it.

### 9. [Low] Composer foot-gun: "2+ lines auto-spawns parallel agents"

A normal multi-line prompt/paste silently fans out into parallel agents. Easy to trigger by
accident. _Enhancement:_ make multi-agent fan-out explicit/opt-in, or confirm before
spawning N agents.

---

## Module-linking / navigation gaps (the "fill the automation gaps" ask)

- **Project → coder:** works, but only after the project exists; no single flow from "I want
  a new Expo app" to a coder session on it.
- **Coder → commit:** the coder leaves changes as "Agent changes (5 pending) / +3112-47" with
  Agent-changes and Git panels present, but the trial couldn't exercise review→commit because
  the app died — so that linkage is unverified.
- **Project → CI/CD → release:** all three surfaces exist and are linked, but the commands
  aren't Expo-aware and the runs couldn't be confirmed; the "Create release file" generator
  was never executed live.
- **GitHub Actions:** present but needs a linked repo (the trial repo was local-only).

---

## Net assessment

- **Coder: excellent.** Build→feature→test→self-verify, hands-off, no prompts. This alone
  does most of the "developer" work.
- **Project + CI/CD + release: promising but not yet reliable.** The surfaces and
  auto-generation are there, but Expo-unaware commands, the PM/lockfile wall, silent run
  feedback, stale metadata, and the app-stability/rebuild failure meant the automated
  **build/test/release loop could not be completed inside the app** — it had to be validated
  from the command line.
- **Biggest wins for "simplification":** (a) make Actions framework-aware (Expo) and
  PM-overridable, (b) give action runs real visible output/exit state, (c) keep the app
  buildable/restartable. Those three would turn a strong coder demo into a trustworthy
  end-to-end pipeline.

---

### Evidence / repro notes

- Driven over CDP `:9223` with the QA harness pattern (`CDP_PORT` + Playwright). Scripts and
  screenshots in the session scratchpad.
- Project id **201**, coder session **1bc829ef-77fe-44a4-b352-6943c36d0ab1**.
- Ground-truth: `npm run build`/`npm run lint` → Missing script (exit 1); pnpm scripts →
  lockfile-policy rejection; coder in-session `npm test` → 1 passed, `tsc --noEmit` clean.
- Note: a `pnpm install` attempt during verification disturbed the app's `node_modules`; a
  fresh `npm install` (or fixing pnpm policy) restores it. App source is intact.
