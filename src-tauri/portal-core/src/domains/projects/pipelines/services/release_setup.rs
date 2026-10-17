//! Helps a project get a `release:` block, and keeps it honest afterwards.
//!
//! `propose` builds a `.portal/pipeline.yml` from what is on disk: if the project
//! already has release scripts (`npm run release`, `scripts/release*.sh`) it wraps
//! those, otherwise it emits per-stack build steps. `health` reports what would make
//! a release fail or quietly skip something. `refine_prompt` / `extract_yaml` frame
//! the optional AI pass; the caller validates the reply with `pipeline_file::parse`.

use super::pipeline_file::{self, FileAction, FileStep, FileWorkflow, PipelineFile, ReleaseSpec};
use crate::domains::projects::modules::detect::{self, DetectedModule};
use crate::process_ext::NoWindowExt;
use serde::Serialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct Proposal {
    pub yaml: String,
    /// `scripts` when it wraps existing release scripts, `generated` otherwise.
    pub source: &'static str,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Issue {
    /// `error` stops a release; `warn` is worth a look.
    pub level: &'static str,
    pub message: String,
}

fn action(run: &str, cwd: Option<&str>, long: bool) -> FileAction {
    FileAction {
        name: None,
        run: run.to_string(),
        cwd: cwd.map(String::from),
        env: HashMap::new(),
        long_running: long,
    }
}

fn steps(ids: &[String], after: &[String]) -> Vec<FileStep> {
    ids.iter()
        .map(|id| FileStep { action: id.clone(), needs: after.to_vec() })
        .collect()
}

/// `(check, build)` commands for a stack, or `None` if there is nothing to run.
fn stack_commands(stack: &str) -> Option<(Option<&'static str>, &'static str)> {
    Some(match stack {
        "flutter" => (Some("flutter analyze"), "flutter build apk --release"),
        "dart" => (Some("dart analyze"), "dart pub get"),
        "nestjs" | "node" | "nextjs" | "sveltekit" | "svelte" | "nuxt" | "vue" | "react" | "angular" => {
            (Some("npm test --if-present"), "npm run build --if-present")
        }
        "android" | "gradle" => (None, if cfg!(windows) { "gradlew assembleRelease" } else { "./gradlew assembleRelease" }),
        "rust" | "tauri" => (Some("cargo test"), "cargo build --release"),
        "go" => (Some("go test ./..."), "go build ./..."),
        "java" => (None, "mvn -B package"),
        _ => return None,
    })
}

/// Scripts the project already has for releasing: `npm run <name>` entries and `scripts/` files.
fn release_scripts(root: &Path) -> (Vec<(String, String)>, Vec<String>) {
    let looks_like_release = |n: &str| ["release", "publish", "deploy", "ship"].iter().any(|k| n.contains(k));
    let mut npm = Vec::new();
    if let Some(pkg) = fs::read_to_string(root.join("package.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
    {
        if let Some(scripts) = pkg.get("scripts").and_then(|s| s.as_object()) {
            for (name, body) in scripts {
                if looks_like_release(name) {
                    npm.push((name.clone(), body.as_str().unwrap_or_default().to_string()));
                }
            }
        }
    }
    let mut files = Vec::new();
    if let Ok(rd) = fs::read_dir(root.join("scripts")) {
        for e in rd.flatten() {
            let n = e.file_name().to_string_lossy().to_string();
            if looks_like_release(&n) {
                files.push(n);
            }
        }
    }
    files.sort();
    (npm, files)
}

pub fn propose(root: &Path, modules: &[DetectedModule]) -> Result<Proposal, String> {
    let mut file = PipelineFile::default();
    let mut gates: Vec<String> = Vec::new();
    let mut kinds: BTreeSet<String> = modules.iter().map(|m| m.stack.clone()).collect();
    if modules.is_empty() {
        if let Some(stack) = detect::classify(root) {
            kinds.insert(stack);
        }
    }

    // One module per entry; a project with no sub-modules is its own single module.
    let targets: Vec<(String, Option<String>, String)> = if modules.is_empty() {
        kinds.iter().map(|s| (s.clone(), None, s.clone())).collect()
    } else {
        modules
            .iter()
            .filter(|m| m.role != "package")
            .map(|m| (m.name.clone(), Some(m.rel_path.clone()), m.stack.clone()))
            .collect()
    };
    let (npm, script_files) = release_scripts(root);
    let ship_npm = npm.iter().find(|(n, _)| n == "release").or_else(|| npm.first());
    // An existing release script already builds and gates everything; adding
    // per-module steps beside it would run the same work twice.
    let wrap_scripts = ship_npm.is_some();
    let mut builds: Vec<String> = Vec::new();
    for (name, cwd, stack) in targets.iter().filter(|_| !wrap_scripts) {
        let Some((check, build)) = stack_commands(stack) else { continue };
        if let Some(check) = check {
            let id = format!("check-{name}");
            file.actions.insert(id.clone(), action(check, cwd.as_deref(), false));
            gates.push(id);
        }
        let id = format!("build-{name}");
        file.actions.insert(id.clone(), action(build, cwd.as_deref(), false));
        builds.push(id);
    }

    let source;
    let (ship_steps, rehearse_steps);
    if let Some((name, _)) = ship_npm {
        source = "scripts";
        file.actions.insert("release".into(), action(&format!("npm run {name}"), None, true));
        let dry = npm.iter().find(|(n, _)| n.contains("dry")).map(|(n, _)| n.clone());
        let mut ship = steps(&gates, &[]);
        ship.push(FileStep { action: "release".into(), needs: gates.clone() });
        ship_steps = ship;
        rehearse_steps = dry.map(|d| {
            file.actions.insert("release-dry".into(), action(&format!("npm run {d}"), None, true));
            let mut s = steps(&gates, &[]);
            s.push(FileStep { action: "release-dry".into(), needs: gates.clone() });
            s
        });
    } else {
        source = "generated";
        let mut ship = steps(&gates, &[]);
        ship.extend(steps(&builds, &gates));
        rehearse_steps = Some(ship.clone());
        ship_steps = ship;
    }

    if ship_steps.is_empty() {
        return Err("Nothing to build here: no known project type was found.".into());
    }
    file.workflows.insert("ship".into(), FileWorkflow { name: Some("Ship".into()), steps: ship_steps });
    let rehearse = rehearse_steps.map(|s| {
        file.workflows.insert("rehearse".into(), FileWorkflow { name: Some("Rehearse (no publish)".into()), steps: s });
        "rehearse".to_string()
    });
    file.release = Some(ReleaseSpec {
        kind: Some(kinds.iter().cloned().collect::<Vec<_>>().join("+")),
        ship: "ship".into(),
        rehearse,
        bump: None,
        env: if source == "scripts" { HashMap::from([("PORTAL_PROGRESS".to_string(), "plain".to_string())]) } else { HashMap::new() },
    });

    let summary = if source == "scripts" {
        format!("Wraps your existing release script{}.", if script_files.is_empty() { String::new() } else { format!(" ({})", script_files.join(", ")) })
    } else {
        format!("Builds {} module(s); add a publish step when you know where it should go.", builds.len())
    };
    Ok(Proposal { yaml: pipeline_file::to_yaml(&file)?, source, summary })
}

fn read_capped(path: &Path) -> String {
    fs::read_to_string(path).map(|s| s.chars().take(200_000).collect()).unwrap_or_default()
}

/// Text of the scripts a release reaches: `npm run <x>` bodies from package.json and
/// files named in commands or inside those files (`source apps.sh`, `node x.js`), a few
/// hops out. Unrelated files in `scripts/` are deliberately left out, or every module
/// name would "appear" somewhere and nothing could ever be flagged.
fn reachable_text(root: &Path, seed: &str) -> String {
    let file_re = regex::Regex::new(r"[\w./\-]+\.(?:sh|js|mjs|cjs|ps1|py|bat|cmd)").unwrap();
    let npm_re = regex::Regex::new(r"npm run ([\w:.-]+)").unwrap();
    let package = fs::read_to_string(root.join("package.json"))
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok());
    let mut out = String::new();
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut frontier = vec![seed.to_string()];
    for _ in 0..4 {
        let mut next = Vec::new();
        for text in &frontier {
            for c in npm_re.captures_iter(text) {
                let name = c[1].to_string();
                if seen.insert(format!("npm:{name}")) {
                    if let Some(body) = package.as_ref().and_then(|p| p["scripts"][&name].as_str()) {
                        next.push(body.to_string());
                    }
                }
            }
            for m in file_re.find_iter(text) {
                let rel = m.as_str().trim_start_matches("./").replace('\\', "/");
                let file = rel.rsplit('/').next().unwrap_or(&rel).to_string();
                // Scripts live under scripts/ or at the root; bare names (apps.sh) are looked up in scripts/.
                for candidate in [root.join(&rel), root.join("scripts").join(&file)] {
                    if candidate.is_file() && seen.insert(candidate.to_string_lossy().into_owned()) {
                        let body = read_capped(&candidate);
                        out.push_str(&body);
                        out.push(' ');
                        next.push(body);
                        break;
                    }
                }
            }
        }
        frontier = next;
    }
    out
}

/// What would make a release fail or quietly skip something.
pub fn health(root: &Path, modules: &[DetectedModule]) -> Vec<Issue> {
    let file = match pipeline_file::load(&root.to_string_lossy()) {
        Ok(Some(f)) => f,
        Ok(None) => return vec![],
        Err(e) => return vec![Issue { level: "error", message: e }],
    };
    let Some(release) = file.release.clone() else { return vec![] };
    let mut issues = Vec::new();

    // Text a release can reach: the commands themselves, and the scripts they call.
    let mut corpus = String::new();
    let mut commands: Vec<&str> = Vec::new();
    for id in std::iter::once(&release.ship).chain(release.rehearse.iter()) {
        if let Some(wf) = file.workflows.get(id) {
            for step in &wf.steps {
                if let Some(a) = file.actions.get(&step.action) {
                    commands.push(&a.run);
                    corpus.push_str(&a.run);
                    corpus.push(' ');
                    if let Some(cwd) = &a.cwd {
                        corpus.push_str(cwd);
                        corpus.push(' ');
                    }
                }
            }
        }
    }
    corpus.push_str(&reachable_text(root, &corpus.clone()));

    let missing: Vec<&str> = modules
        .iter()
        .filter(|m| m.role != "package" && !corpus.contains(&m.name) && !corpus.contains(&m.rel_path))
        .map(|m| m.name.as_str())
        .collect();
    if !missing.is_empty() {
        issues.push(Issue { level: "warn", message: format!("Not mentioned by the release: {}", missing.join(", ")) });
    }

    let mut tools: BTreeMap<&str, ()> = BTreeMap::new();
    for c in &commands {
        if let Some(first) = c.split_whitespace().next() {
            if !["echo", "cd", "set", "exit", "call"].contains(&first) {
                tools.insert(first, ());
            }
        }
    }
    for t in ["gcloud", "gh", "flutter", "docker", "firebase"] {
        if corpus.split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_').any(|w| w == t) {
            tools.insert(t, ());
        }
    }
    let finder = if cfg!(windows) { "where" } else { "which" };
    let absent: Vec<&str> = tools
        .keys()
        .copied()
        .filter(|t| !t.starts_with('.') && !t.contains('/') && !t.contains('\\'))
        .filter(|t| {
            !std::process::Command::new(finder)
                .no_window()
                .arg(t)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        })
        .collect();
    if !absent.is_empty() {
        issues.push(Issue { level: "warn", message: format!("Not installed or not on PATH: {}", absent.join(", ")) });
    }
    issues
}

/// `(system, user)` prompts asking a model to edit the pipeline file.
pub fn refine_prompt(current_yaml: &str, instruction: &str, modules: &[DetectedModule]) -> (String, String) {
    let system = "You edit a Portal Desktop `.portal/pipeline.yml`. Reply with the complete YAML file only: no prose, no code fences.\n\
Schema: `version: 1`; `release: {kind, ship, rehearse, bump, env}` where ship and rehearse name workflows; \
`actions: {<id>: {name, run, cwd, env, longRunning}}` where `run` is one shell command, `cwd` is relative to the project root; \
`workflows: {<id>: {name, steps: [{action, needs: [action ids]}]}}`. \
`rehearse` must never publish or deploy anything. Keep existing action ids unless the request says otherwise. \
Commands run through `cmd /C` on Windows and `sh -c` elsewhere, one command per action.".to_string();
    let mods: Vec<String> = modules.iter().map(|m| format!("{} ({}, {})", m.rel_path, m.stack, m.role)).collect();
    let user = format!(
        "Modules found: {}\n\nCurrent file:\n{}\n\nRequest: {}",
        if mods.is_empty() { "none".into() } else { mods.join("; ") },
        current_yaml,
        instruction
    );
    (system, user)
}

/// The YAML in a model reply, whether or not it wrapped it in a code fence.
pub fn extract_yaml(reply: &str) -> String {
    let t = reply.trim();
    if let Some(start) = t.find("```") {
        let after = &t[start + 3..];
        let after = after.strip_prefix("yaml").or_else(|| after.strip_prefix("yml")).unwrap_or(after);
        if let Some(end) = after.find("```") {
            return after[..end].trim().to_string();
        }
    }
    t.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(rel: &str, stack: &str, role: &str) -> DetectedModule {
        DetectedModule { rel_path: rel.into(), name: rel.rsplit('/').next().unwrap().into(), stack: stack.into(), role: role.into() }
    }

    #[test]
    fn generated_proposal_parses_and_wraps_scripts_when_present() {
        let dir = std::env::temp_dir().join(format!("portal-release-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let mods = [module("apps/gym", "flutter", "app"), module("server/api", "nestjs", "service")];

        let p = propose(&dir, &mods).unwrap();
        assert_eq!(p.source, "generated");
        let file = pipeline_file::parse(&p.yaml).unwrap();
        assert_eq!(file.release.unwrap().ship, "ship");
        assert!(file.actions["build-gym"].cwd.as_deref() == Some("apps/gym"));

        fs::write(dir.join("package.json"), r#"{"scripts":{"release":"x","release:dry":"x --dry-run"}}"#).unwrap();
        let p = propose(&dir, &mods).unwrap();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(p.source, "scripts");
        assert!(pipeline_file::parse(&p.yaml).unwrap().actions.contains_key("release-dry"));
    }

    #[test]
    fn extracts_fenced_yaml() {
        assert_eq!(extract_yaml("Here:\n```yaml\nversion: 1\n```\nbye"), "version: 1");
        assert_eq!(extract_yaml("version: 1"), "version: 1");
    }

    #[test]
    fn health_follows_release_scripts_and_flags_unmentioned_modules() {
        let dir = std::env::temp_dir().join(format!("portal-health-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join(".portal")).unwrap();
        fs::create_dir_all(dir.join("scripts")).unwrap();
        fs::write(dir.join("package.json"), r#"{"scripts":{"release":"node scripts/run.js"}}"#).unwrap();
        fs::write(dir.join("scripts/run.js"), "run('scripts/apps.sh')").unwrap();
        fs::write(dir.join("scripts/apps.sh"), "APPS=(gym task)").unwrap();
        fs::write(dir.join("scripts/unrelated.sh"), "mail").unwrap();
        fs::write(
            dir.join(".portal/pipeline.yml"),
            "version: 1
release: {ship: s}
actions:
  r: {run: echo ok}
workflows:
  s: {steps: [{action: r}]}
",
        )
        .unwrap();
        fs::write(
            dir.join(".portal/pipeline.yml"),
            "version: 1
release: {ship: s}
actions:
  r: {run: npm run release}
workflows:
  s: {steps: [{action: r}]}
",
        )
        .unwrap();
        let mods = [module("apps/gym", "flutter", "app"), module("apps/task", "flutter", "app"), module("apps/mail", "flutter", "app")];

        let issues = health(&dir, &mods);
        let _ = fs::remove_dir_all(&dir);
        let drift = issues.iter().find(|i| i.message.starts_with("Not mentioned")).unwrap();
        // gym and task come from apps.sh, two hops from the release; mail only appears in an unrelated script.
        assert!(drift.message.ends_with("mail"), "{}", drift.message);
    }
}
